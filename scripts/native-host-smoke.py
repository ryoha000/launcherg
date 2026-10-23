"""Exercise release Native Messaging framing, both stores, and archive failure recovery.

All database/files live in an isolated temporary data directory. No account is used.
"""
from contextlib import ExitStack, closing
import json
import os
from pathlib import Path
import sqlite3
import struct
import subprocess
import tempfile
import zipfile

host = Path(os.environ.get('LAUNCHERG_SMOKE_HOST', 'src-tauri/resources/integrations/native-messaging-host.exe')).resolve()
assert host.is_file(), 'Build and stage the native host first'

with tempfile.TemporaryDirectory(prefix='launcherg-smoke-') as temporary, ExitStack() as cleanup:
    root = Path(temporary)
    env = {**os.environ, 'LAUNCHERG_DATA_DIR': str(root / 'data')}

    def send(case, value, request_id):
        payload = json.dumps({'request_id': request_id, 'message': {'case': case, 'value': value}}).encode()
        result = subprocess.run([str(host)], input=struct.pack('<I', len(payload)) + payload,
                                capture_output=True, env=env, timeout=60, check=True)
        assert len(result.stdout) >= 4, result.stderr.decode(errors='replace')[-2000:]
        length = struct.unpack('<I', result.stdout[:4])[0]
        assert len(result.stdout) == length + 4, 'Non-protocol data appeared on stdout'
        return json.loads(result.stdout[4:])

    for store, case, game in [
        ('DMM', 'SyncDmmGames', {'id': 'release-dmm-test', 'category': 'digital', 'subcategory': 'pcgame'}),
        ('DLsite', 'SyncDlsiteGames', {'id': 'RJ-release-test', 'category': 'maniax'}),
    ]:
        game.update(title='Release integration test ' + store, image_url='')
        result = send(case, {'extension_id': 'release-test', 'games': [game]}, 'sync-' + store)
        assert result['success'], result
        again = send(case, {'extension_id': 'release-test', 'games': [game]}, 'sync-again-' + store)
        assert again['success'] and again['response']['value']['new_count'] == 0, again
    database = root / 'data' / 'launcherg_sqlite.db3'
    assert database.is_file(), 'Native host did not honor the isolated data directory'
    connection = cleanup.enter_context(closing(sqlite3.connect(database)))
    assert connection.execute('SELECT COUNT(*) FROM works').fetchone()[0] == 2

    for store, store_id, intent_case in [('DMM', 'release-dmm-test', 'Dmm'), ('DLsite', 'RJ-release-test', 'Dlsite')]:
        archive = root / (store + ' game.zip')
        with zipfile.ZipFile(archive, 'w') as package:
            package.writestr('game/game.exe', b'MZ fixture; never executed')
        intent = {'case': intent_case, 'value': {'game_store_id': store_id, 'game_category': 'digital' if store == 'DMM' else 'maniax'}}
        if store == 'DMM':
            intent['value']['game_subcategory'] = 'pcgame'
        value = {'extension_id': 'release-test', 'intent': intent, 'items': [{'id': 1, 'filename': str(archive)}]}
        result = send('DownloadsCompleted', value, 'download-' + store)
        assert result['success'], result
        before = connection.execute('SELECT COUNT(*) FROM work_download_paths').fetchone()[0]
        assert send('DownloadsCompleted', value, 'download-' + store)['success']
        assert connection.execute('SELECT COUNT(*) FROM work_download_paths').fetchone()[0] == before
        assert archive.is_file(), 'Original archive was unexpectedly removed'

    invalid = root / 'broken.zip'
    invalid.write_bytes(b'not a zip file')
    value['items'] = [{'id': 2, 'filename': str(invalid)}]
    assert not send('DownloadsCompleted', value, 'broken-archive')['success']
    assert invalid.is_file()
    # Repair the same failed download, then retry the same request.
    with zipfile.ZipFile(invalid, 'w') as package:
        package.writestr('game.exe', b'MZ fixture; never executed')
    assert send('DownloadsCompleted', value, 'broken-archive')['success']

    non_game = root / 'empty-game.zip'
    with zipfile.ZipFile(non_game, 'w') as package:
        package.writestr('readme.txt', 'No launchable executable')
    value['items'] = [{'id': 3, 'filename': str(non_game)}]
    assert not send('DownloadsCompleted', value, 'no-executable')['success']
    assert non_game.is_file()

    # Legacy Windows ZIPs use CP932 names without the UTF-8 bit.
    japanese = root / 'japanese.zip'
    with zipfile.ZipFile(japanese, 'w') as package:
        package.writestr('aaaa.exe', b'MZ fixture; never executed')
    japanese.write_bytes(japanese.read_bytes().replace(b'aaaa.exe', '日本.exe'.encode('cp932')))
    value['items'] = [{'id': 6, 'filename': str(japanese)}]
    assert send('DownloadsCompleted', value, 'japanese-name')['success']
    assert list((root / 'data').rglob('日本.exe')), 'Japanese filename was corrupted'

    traversal = root / 'unsafe.zip'
    with zipfile.ZipFile(traversal, 'w') as package:
        package.writestr('../escape.exe', b'fixture')
    value['items'] = [{'id': 7, 'filename': str(traversal)}]
    assert not send('DownloadsCompleted', value, 'unsafe-path')['success']
    assert not list((root / 'data').rglob('escape.exe'))

    failed_exe = root / 'failed-extractor.exe'
    fixture_source = root / 'failed-extractor.cs'
    fixture_source.write_text('public class Fixture { public static int Main(string[] args) { return 1; } }')
    compiler = Path(os.environ['WINDIR']) / 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
    subprocess.run([str(compiler), '/nologo', '/target:exe', '/out:' + str(failed_exe), str(fixture_source)],
                   check=True, capture_output=True)
    part = root / 'part.bin'
    part.write_bytes(b'fixture')
    value['items'] = [{'id': 4, 'filename': str(failed_exe)}, {'id': 5, 'filename': str(part)}]
    assert not send('DownloadsCompleted', value, 'failed-split')['success']
    assert failed_exe.is_file() and part.is_file()
    assert connection.execute('PRAGMA foreign_key_check').fetchall() == []
    connection.close()
    print('PASS: DMM/DLsite sync, duplicate sync, ZIP install, replay, corrupt ZIP recovery, missing executable, failed split extraction, original-file retention')
