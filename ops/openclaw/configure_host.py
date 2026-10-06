"""Approved narrow host setup. Secrets travel only over SSH stdin."""
import json,pathlib,subprocess,sys
import boto3
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[2]/'Harness'))
from harness.config import resolve_jev_key
root=pathlib.Path(__file__).resolve().parent
proof=json.loads((root/'preservation.json').read_text())
if not proof.get('restored_database_integrity'):raise SystemExit('Verified backup required')
ec2=boto3.client('ec2',region_name='us-east-1')
i=ec2.describe_instances(InstanceIds=[proof['instance']])['Reservations'][0]['Instances'][0]
ssh=['ssh','-i',str(pathlib.Path.home()/'.ssh/openclaw-key.pem'),'-o','BatchMode=yes','-o','StrictHostKeyChecking=yes','ec2-user@'+i['PublicIpAddress']]
key=resolve_jev_key()
if not key:raise SystemExit('Native Jev key unavailable')
try:
 ec2.modify_volume(VolumeId=proof['volume'],Size=40)
 volume='expansion_requested'
except Exception as e:volume=getattr(e,'response',{}).get('Error',{}).get('Code',type(e).__name__)
code=r'''
import json,os,pathlib,subprocess,urllib.request
h=pathlib.Path.home();out={}
key=__KEY__
p=h/'.config/harness/jev.env';p.parent.mkdir(parents=True,exist_ok=True)
fd=os.open(p,os.O_WRONLY|os.O_CREAT|os.O_TRUNC,0o600)
with os.fdopen(fd,'w') as f:f.write('JEV_API_KEY='+key+'\n')
os.chmod(p,0o600)
for p in [h/'.openclaw/env.vars',h/'.config/harness/openrouter.env',h/'.openclaw/openclaw.json']:
 if p.exists():os.chmod(p,0o600)
out['credential_modes_restricted']=True
out['native_jev_file_written']=True
out['volume_request']=__VOLUME__
if __VOLUME__=='expansion_requested':
 for cmd in [['sudo','growpart','/dev/nvme0n1','1'],['sudo','xfs_growfs','/']]:
  p=subprocess.run(cmd,capture_output=True,text=True);out[cmd[1]]={'exit':p.returncode,'result':(p.stdout+p.stderr)[-300:]}
else:
 config=(h/'.openclaw/openclaw.json').read_text()
 with urllib.request.urlopen('http://127.0.0.1:11434/api/ps',timeout=5) as r:active=json.load(r)['models']
 with urllib.request.urlopen('http://127.0.0.1:11434/api/tags',timeout=5) as r:models=json.load(r)['models']
 unused=next((m for m in models if m['name']=='gemma3n:e2b'),None)
 if unused and not active and 'gemma3n:e2b' not in config:
  out['removed_unused_model']={k:unused.get(k) for k in ['name','digest','size']}
  out['model_restore']='ollama pull gemma3n:e2b'
  p=subprocess.run(['ollama','rm','gemma3n:e2b'],capture_output=True,text=True,timeout=40)
  out['model_cleanup_exit']=p.returncode
out['disk']=subprocess.check_output(['df','-h','/'],text=True).strip()
print(json.dumps(out))
'''.replace('__KEY__',repr(key)).replace('__VOLUME__',repr(volume))
p=subprocess.run(ssh+['python3 -'],input=code,text=True,capture_output=True,timeout=90)
if p.returncode:raise SystemExit('Host setup failed: '+p.stderr[-200:])
result=json.loads(p.stdout);(root/'host-setup.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
