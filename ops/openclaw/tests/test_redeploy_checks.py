import ast
import pathlib
import subprocess
import unittest
from unittest.mock import Mock

# Load only the pure probe contract: importing the deployment entrypoint creates AWS clients.
tree=ast.parse((pathlib.Path(__file__).resolve().parents[1]/'redeploy_openclaw.py').read_text())
scope={}
exec(compile(ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='ensure'],type_ignores=[]),'<probe-contract>','exec'),scope)
ensure=scope['ensure']
class ProbeTests(unittest.TestCase):
    def test_failed_completed_process_requires_repair_and_recheck(self):
        cond=Mock(side_effect=[subprocess.CompletedProcess([],1),subprocess.CompletedProcess([],0)])
        fix=Mock(return_value=subprocess.CompletedProcess([],0))
        self.assertTrue(ensure(cond,'service',fix));fix.assert_called_once();self.assertEqual(cond.call_count,2)
    def test_failed_repair_stops(self):
        with self.assertRaises(RuntimeError):ensure(lambda:False,'service',lambda:subprocess.CompletedProcess([],1))
    def test_false_success_repair_stops(self):
        with self.assertRaises(RuntimeError):ensure(lambda:False,'service',lambda:None)
    def test_healthy_probe_does_not_mutate(self):
        fix=Mock();self.assertTrue(ensure(lambda:True,'service',fix));fix.assert_not_called()
if __name__=='__main__':unittest.main()
