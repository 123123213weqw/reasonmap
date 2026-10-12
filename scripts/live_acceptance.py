#!/usr/bin/env python3
"""Opt-in synthetic live checks; keys are read from stdin/environment, never argv/logs.

A 200 response does NOT prove that the provider honored an effort level. This
runner records request acceptance and observable reasoning separately. It does
not evaluate reasoning quality or claim high > low. No response bodies are saved.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import sys
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PROMPT = 'Compute 17*19. Reply with only the decimal number.'

def cases(catalog, family, base):
    result = []
    for route in catalog['routes']:
        if route['family'] != family or base not in route['base_urls']:
            continue
        profile = catalog['profiles'][route['profile']]
        for model in route['models']:
            legacy = catalog['identities'][model]['legacy_alias']
            efforts = [e for e in profile['efforts'] if not legacy or e['id'] == profile['default_effort']]
            for effort in efforts:
                result.append((route['api'], model, effort['id'], effort['request_patch']))
            if not legacy:
                default = next(e for e in profile['efforts'] if e['id'] == profile['default_effort'])
                result.append((route['api'], model, 'catalog-default', default['request_patch']))
    return result

def request(base, path, key, payload=None):
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(base + path, data=data, headers={
        'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=60) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        # Never retain free-form error text/headers; a numeric provider code can
        # distinguish quota from concurrency limits without exposing a body.
        try:
            body = json.loads(error.read(65536))
            code = str((body.get('error') or {}).get('code', ''))
        except Exception:
            code = ''
        return error.code, {'numeric_error_code': code if re.fullmatch(r'[0-9]{3,6}', code) else None}
    except Exception:
        return None, {}

def observe(protocol, body):
    usage = body.get('usage') or {}
    reported = {k: v for k, v in usage.items() if k in (
        'prompt_tokens', 'completion_tokens', 'total_tokens', 'input_tokens', 'output_tokens')
        and isinstance(v, int) and not isinstance(v, bool)}
    details = usage.get('output_tokens_details') or usage.get('completion_tokens_details') or {}
    reasoning_tokens = details.get('reasoning_tokens')
    if not isinstance(reasoning_tokens, int) or isinstance(reasoning_tokens, bool):
        reasoning_tokens = None
    if protocol == 'openai-completions':
        choice = (body.get('choices') or [{}])[0]
        message = choice.get('message') or {}
        answer = message.get('content') or ''
        thought = bool(message.get('reasoning_content'))
        # Only classify a clean terminator, never a truncated answer as success.
        completed = choice.get('finish_reason') == 'stop'
        observable = 'reasoning_content' in message
    else:
        outputs = body.get('output') or []
        answer = ''.join(c.get('text', '') for item in outputs if item.get('type') == 'message'
                         for c in item.get('content', []) if c.get('type') == 'output_text')
        thought = any(item.get('type') == 'reasoning' for item in outputs)
        completed = body.get('status') == 'completed'
        observable = thought or reasoning_tokens is not None
    if reasoning_tokens is not None:
        thought = thought or reasoning_tokens > 0
    return {'answer_correct': isinstance(answer, str) and answer.strip() == '323',
            'completed': completed, 'reasoning_observed': thought,
            'reasoning_observable': observable, 'reasoning_tokens': reasoning_tokens,
            'usage': reported}

def check_case(base, key, case):
    protocol, model, effort, patch = case
    payload = {'model': model, 'stream': False}
    if protocol == 'openai-completions':
        payload.update(messages=[{'role': 'user', 'content': PROMPT}], max_tokens=2048)
        path = '/chat/completions'
    else:
        payload.update(input=[{'role': 'user', 'content': PROMPT}], max_output_tokens=2048)
        path = '/responses'
    payload.update(patch)
    start = time.monotonic()
    status, body = request(base, path, key, payload)
    observation = observe(protocol, body) if status == 200 else {}
    accepted = status == 200 and observation.get('answer_correct', False) and observation.get('completed', False)
    toggle = 'not-observable'
    if accepted and observation['reasoning_observable']:
        expected = effort != 'off'
        toggle = 'pass' if observation['reasoning_observed'] == expected else 'fail'
    return {'protocol': protocol, 'model': model, 'effort': effort, 'http_status': status,
            'elapsed_ms': round((time.monotonic() - start) * 1000),
            'numeric_error_code': body.get('numeric_error_code'),
            'request_acceptance': 'pass' if accepted else 'fail',
            'reasoning_toggle_observation': toggle, **observation}

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--family', choices=['deepseek', 'glm'], required=True)
    p.add_argument('--base-url', required=True)
    p.add_argument('--key-stdin', action='store_true')
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--workers', type=int, choices=[1, 2], default=1)
    p.add_argument('--retry-failed', type=Path, help='Recheck failed cases only; preserves original attempts in the report')
    args = p.parse_args()
    data = json.loads((ROOT / 'data/catalog.json').read_text())
    selected = cases(data, args.family, args.base_url)
    if not selected:
        p.error('Use an exact official base URL from the catalog')
    previous = None
    if args.retry_failed:
        previous = json.loads(args.retry_failed.read_text())
        if previous.get('family') != args.family or previous.get('base_url') != args.base_url or previous.get('catalog_version') != data['version']:
            p.error('Retry evidence must match this family, exact endpoint and catalog version')
        failed = {(r['protocol'], r['model'], r['effort']) for r in previous['cases']
                  if r['request_acceptance'] != 'pass' or r['reasoning_toggle_observation'] == 'fail'}
        selected = [c for c in selected if c[:3] in failed]
        if not selected:
            p.error('No failed cases to retry')
    key = sys.stdin.readline().strip() if args.key_stdin else os.environ.get('CATALOG_API_KEY', '').strip()
    if not key:
        p.error('Missing key; supply stdin or CATALOG_API_KEY, never a command argument')
    # Do not forward a credential to any other endpoint after authentication failure.
    status, models = request(args.base_url, '/models', key)
    if status in (401, 403):
        report = {'schema_version': 1, 'family': args.family, 'catalog_version': data['version'],
                  'checked_at': datetime.now(timezone.utc).isoformat(),
                  'status': 'blocked-auth', 'models_http_status': status, 'cases': []}
    else:
        with ThreadPoolExecutor(max_workers=args.workers) as pool:
            rows = list(pool.map(lambda c: check_case(args.base_url, key, c), selected))
        if previous:
            retried = { (r['protocol'], r['model'], r['effort']): r for r in rows }
            merged = []
            for old in previous['cases']:
                identity = (old['protocol'], old['model'], old['effort'])
                new = retried.get(identity)
                if new:
                    history = old.get('prior_attempts', []) + [{k:v for k,v in old.items() if k != 'prior_attempts'}]
                    new['prior_attempts'] = history
                    merged.append(new)
                else:
                    merged.append(old)
            rows = merged
        report = {'schema_version': 1, 'family': args.family, 'catalog_version': data['version'],
                  'checked_at': datetime.now(timezone.utc).isoformat(), 'base_url': args.base_url,
                  'models_http_status': status, 'models': [
                      {'id': m.get('id'), 'name': m.get('name'), 'effort': m.get('effort')}
                      for m in models.get('data', []) if m.get('id') in data['identities']],
                  'limits': {'max_output_tokens_per_request': 2048, 'max_workers': args.workers},
                  'cases': rows, 'status': 'pass' if all(r['request_acceptance'] == 'pass'
                      and r['reasoning_toggle_observation'] != 'fail' for r in rows) else 'fail',
                  'quality_ranking_verified': False}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'cases': len(report['cases']),
                      'acceptance_passed': sum(r['request_acceptance'] == 'pass' for r in report['cases'])}))
    return 0 if report['status'] == 'pass' else 1

if __name__ == '__main__':
    raise SystemExit(main())
