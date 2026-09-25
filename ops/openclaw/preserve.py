"""Preserve OpenClaw application state off-host, encrypted with Windows DPAPI."""
import ctypes, datetime, hashlib, io, json, pathlib, sqlite3, subprocess, tarfile, tempfile
import boto3

ROOT=pathlib.Path(__file__).resolve().parent
ec2=boto3.client('ec2',region_name='us-east-1')
nodes=[i for r in ec2.describe_instances(Filters=[{'Name':'tag:Name','Values':['openclaw-node']},{'Name':'instance-state-name','Values':['running']}])['Reservations'] for i in r['Instances']]
if len(nodes)!=1: raise SystemExit('Expected one running OpenClaw instance')
node=nodes[0]; iid=node['InstanceId']; vol=node['BlockDeviceMappings'][0]['Ebs']['VolumeId']
ssh=['ssh','-i',str(pathlib.Path.home()/'.ssh/openclaw-key.pem'),'-o','BatchMode=yes','-o','StrictHostKeyChecking=yes','-o','ConnectTimeout=15','ec2-user@'+node['PublicIpAddress']]
code=r'''
import io,os,pathlib,sqlite3,sys,tarfile,tempfile
h=pathlib.Path.home()
roots=['.config/harness','.config/systemd/user','.config/scmessenger','.local/share/scmessenger','.openclaw/openclaw.json','.openclaw/env.vars','.openclaw/state','.openclaw/agents','.openclaw/workspace','.openclaw/scm-bridge-allowlist.json','.openclaw/scm-bridge-seen.json','scm_bridge.py']
with tempfile.TemporaryDirectory(prefix='oc-preserve-') as td, tarfile.open(fileobj=sys.stdout.buffer,mode='w|gz') as tf:
 for rel in roots:
  p=h/rel
  if not p.exists():continue
  files=[p] if p.is_file() else [x for x in p.rglob('*') if x.is_file() and not x.is_symlink()]
  for f in files:
   if '.git' in f.parts or f.name.endswith(('-wal','-shm')):continue
   if f.stat().st_size>30*1024*1024:continue
   with f.open('rb') as inp:header=inp.read(16)
   if header==b'SQLite format 3\0':
    dest=pathlib.Path(td)/'db'
    with sqlite3.connect(f.as_uri()+'?mode=ro',uri=True) as src, sqlite3.connect(str(dest)) as dst:src.backup(dst)
    tf.add(dest,arcname=str(f.relative_to(h)))
    dest.unlink()
   else:tf.add(f,arcname=str(f.relative_to(h)),recursive=False)
'''
p=subprocess.run(ssh+['python3 -'],input=code.encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120)
if p.returncode:raise SystemExit('Remote archive failed; '+p.stderr.decode()[:200])
data=p.stdout
class Blob(ctypes.Structure):_fields_=[('size',ctypes.c_ulong),('data',ctypes.POINTER(ctypes.c_ubyte))]
def protect(raw,decode=False):
 buf=ctypes.create_string_buffer(raw); src=Blob(len(raw),ctypes.cast(buf,ctypes.POINTER(ctypes.c_ubyte))); dst=Blob()
 fn=ctypes.windll.crypt32.CryptUnprotectData if decode else ctypes.windll.crypt32.CryptProtectData
 if not fn(ctypes.byref(src),None,None,None,None,1,ctypes.byref(dst)):raise ctypes.WinError()
 try:return ctypes.string_at(dst.data,dst.size)
 finally:ctypes.windll.kernel32.LocalFree(dst.data)
stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
dest=pathlib.Path.home()/'.ssh/oc-backups'/('state-'+stamp+'.tar.gz.dpapi'); dest.parent.mkdir(exist_ok=True)
dest.write_bytes(protect(data)); restored=protect(dest.read_bytes(),True)
if restored!=data:raise RuntimeError('Encrypted backup verification failed')
manifest=[]; databases=[]
with tarfile.open(fileobj=io.BytesIO(restored),mode='r:gz') as tf, tempfile.TemporaryDirectory(prefix='oc-restore-') as td:
 for m in tf.getmembers():
  if not m.isfile():continue
  body=tf.extractfile(m).read(); manifest.append({'path':m.name,'bytes':len(body),'sha256':hashlib.sha256(body).hexdigest()})
  if body.startswith(b'SQLite format 3\0'):
   check=pathlib.Path(td)/'restored.db';check.write_bytes(body)
   db=sqlite3.connect(check)
   try:ok=db.execute('pragma integrity_check').fetchone()[0]
   finally:db.close()
   if ok!='ok':raise RuntimeError('Restored database failed integrity check')
   databases.append(m.name);check.unlink()
result={'instance':iid,'ip':node['PublicIpAddress'],'volume':vol,'backup_file':str(dest),'archive_sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data),'files':len(manifest),'restored_database_integrity':databases,'manifest':manifest,'created_utc':stamp}
for name,fn in [('snapshot',lambda:ec2.create_snapshot(VolumeId=vol,Description='OpenClaw pre-implementation state '+stamp)['SnapshotId']),('termination_protection',lambda:ec2.modify_instance_attribute(InstanceId=iid,DisableApiTermination={'Value':True}))]:
 try:result[name]=fn()
 except Exception as e:result[name]={'unavailable':getattr(e,'response',{}).get('Error',{}).get('Code',type(e).__name__)}
(ROOT/'preservation.json').write_text(json.dumps(result,indent=2))
print(json.dumps({k:v for k,v in result.items() if k!='manifest'}))
