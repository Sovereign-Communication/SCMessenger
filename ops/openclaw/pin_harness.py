"""Deploy an immutable tracked Harness source snapshot; retain old installation."""
import hashlib,io,json,pathlib,subprocess,tarfile
ROOT=pathlib.Path(__file__).resolve().parent
repo=ROOT.parents[1]/'Harness'
sha=subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip()
if subprocess.check_output(['git','-C',str(repo),'status','--porcelain'],text=True).strip():raise SystemExit('Harness checkout must be clean')
archive=subprocess.check_output(['git','-C',str(repo),'archive','--format=tar',sha])
p=json.loads((ROOT/'preservation.json').read_text())
ssh=['ssh','-i',str(pathlib.Path.home()/'.ssh/openclaw-key.pem'),'-o','BatchMode=yes','-o','StrictHostKeyChecking=yes','ec2-user@'+p['ip']]
remote='/home/ec2-user/dogfood/releases/harness-'+sha
r=subprocess.run(ssh+['mkdir -p '+remote+' && tar -xf - -C '+remote],input=archive,capture_output=True,timeout=60)
if r.returncode:raise SystemExit('Source transfer failed')
code='''import hashlib,json,pathlib
root=pathlib.Path(%r)
files={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'harness').glob('*.py')}
print(json.dumps(files))
'''%remote
r=subprocess.run(ssh+['python3 -'],input=code,text=True,capture_output=True,timeout=15)
if r.returncode:raise SystemExit('Source verification failed')
files=json.loads(r.stdout)
with tarfile.open(fileobj=io.BytesIO(archive)) as tf:
 for name,digest in files.items():
  if hashlib.sha256(tf.extractfile(name).read()).hexdigest()!=digest:raise SystemExit('Source digest mismatch')
record={'sha':sha,'remote_path':remote,'archive_sha256':hashlib.sha256(archive).hexdigest(),'verified_module_count':len(files)}
(ROOT/'harness-release.json').write_text(json.dumps(record,indent=2))
print(json.dumps(record))
