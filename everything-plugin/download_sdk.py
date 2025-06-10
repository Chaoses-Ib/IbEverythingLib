# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "requests",
# ]
# ///
import requests
from pathlib import Path
import zipfile

path = Path('../target/sdk/Everything-1.5-Plugin-SDK.zip')
zip = requests.get('https://www.voidtools.com/Everything-1.5-Plugin-SDK.zip')
path.write_bytes(zip.content)
with zipfile.ZipFile(path, 'r') as zip_ref:
    zip_ref.extractall('../target/sdk')

h = Path('../target/sdk/everything_plugin.h')
# TODO: Trailing comments
c = h.read_text().replace('// ', '///').replace('/*', '/**').replace('/** ', '/**')
h.write_text(f'''#include <Windows.h>
{c}''')
