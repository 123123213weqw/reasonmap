"""Offline runner tests: no network and no real credentials."""
import importlib.util
import json
from pathlib import Path
import unittest
import sys
import os
spec = importlib.util.spec_from_file_location('live_acceptance', Path(__file__).with_name('live_acceptance.py'))
live = importlib.util.module_from_spec(spec)
spec.loader.exec_module(live)

class RunnerTests(unittest.TestCase):
    def test_catalog_cases_do_not_forward_to_arbitrary_endpoints(self):
        catalog = json.loads((live.ROOT / 'data/catalog.json').read_text(encoding='utf-8'))
        self.assertEqual(live.cases(catalog, 'glm', 'https://proxy.test'), [])
        cases = live.cases(catalog, 'deepseek', 'https://api.deepseek.com')
        self.assertEqual(len(cases), 24)
        self.assertTrue(all(c[2] == 'high' for c in cases if 'vision-exp' in c[1]))
        self.assertFalse(any(c[1] == 'deepseek-v4.1-flash' for c in cases))

    def test_chat_clean_completion_reasoning_and_usage(self):
        body = {'choices': [{'finish_reason': 'stop', 'message': {'content': '323', 'reasoning_content': 'synthetic'}}], 'usage': {'completion_tokens': 5, 'secret': 'do not preserve'}}
        row = live.observe('openai-completions', body)
        self.assertTrue(row['completed'] and row['answer_correct'] and row['reasoning_observed'])
        self.assertEqual(row['usage'], {'completion_tokens': 5})
        body['choices'][0]['finish_reason'] = 'length'
        self.assertFalse(live.observe('openai-completions', body)['completed'])
        self.assertNotIn('synthetic', json.dumps(row))

    def test_responses_not_observable_is_not_behavior_verified(self):
        body = {'status': 'completed', 'output': [{'type': 'message', 'content': [{'type': 'output_text', 'text': '323'}]}]}
        row = live.observe('openai-responses', body)
        self.assertTrue(row['answer_correct'])
        self.assertFalse(row['reasoning_observable'])
        body['usage'] = {'output_tokens_details': {'reasoning_tokens': 0}}
        row = live.observe('openai-responses', body)
        self.assertTrue(row['reasoning_observable'])
        self.assertFalse(row['reasoning_observed'])

    def test_http_200_wrong_answer_and_truncation_are_failures(self):
        old = live.request
        try:
            for body in [
                {'choices': [{'finish_reason': 'stop', 'message': {'content': '322'}}]},
                {'choices': [{'finish_reason': 'length', 'message': {'content': '323'}}]},
            ]:
                live.request = lambda *args: (200, body)
                result = live.check_case('https://example.test', 'fake', ('openai-completions', 'model', 'high', {}))
                self.assertEqual(result['request_acceptance'], 'fail')
        finally:
            live.request = old


    def test_retry_preserves_first_failure_without_changing_the_scope(self):
        import tempfile
        from unittest.mock import patch
        old = live.request
        try:
            catalog = json.loads((live.ROOT / 'data/catalog.json').read_text(encoding='utf-8'))
            with tempfile.TemporaryDirectory() as tmp:
                prior = Path(tmp) / 'prior.json'
                result = Path(tmp) / 'result.json'
                row = {'protocol': 'openai-completions', 'model': 'deepseek-flash', 'effort': 'low', 'http_status': 429, 'request_acceptance': 'fail', 'reasoning_toggle_observation': 'not-observable'}
                prior.write_text(json.dumps({'family':'deepseek', 'base_url':'https://api.deepseek.com', 'catalog_version':catalog['version'], 'cases':[row]}), encoding='utf-8')
                live.request = lambda *args: (200, {'choices':[{'finish_reason':'stop','message':{'content':'323', 'reasoning_content':'fixture'}}]})
                argv = ['live_acceptance.py','--family','deepseek','--base-url','https://api.deepseek.com','--output',str(result),'--retry-failed',str(prior)]
                original_read = Path.read_text
                original_write = Path.write_text
                def utf8_read(path, *args, **kwargs):
                    self.assertEqual(kwargs.get('encoding'), 'utf-8', 'never depend on Windows locale')
                    return original_read(path, *args, **kwargs)
                def utf8_write(path, *args, **kwargs):
                    self.assertEqual(kwargs.get('encoding'), 'utf-8', 'report encoding must be explicit')
                    return original_write(path, *args, **kwargs)
                with patch.object(sys, 'argv', argv), patch.dict(os.environ, {'CATALOG_API_KEY':'fake'}, clear=True), patch.object(Path, 'read_text', utf8_read), patch.object(Path, 'write_text', utf8_write):
                    self.assertEqual(live.main(), 0)
                merged = json.loads(result.read_text(encoding='utf-8'))['cases'][0]
                self.assertEqual(merged['prior_attempts'][0]['http_status'], 429)
                self.assertEqual(merged['request_acceptance'], 'pass')
                self.assertEqual(merged['model'], 'deepseek-flash')
        finally:
            live.request = old

if __name__ == '__main__':
    unittest.main()
