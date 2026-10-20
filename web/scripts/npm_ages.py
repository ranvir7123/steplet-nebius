"""For every package in an npm package-lock.json (lockfile version 3), print its publish date, its age in days
today, and whether it was deprecated or its version is gone from the registry (npm's ways of withdrawing a
package). One request per second. Ends with a count of packages under 14 days old and of withdrawn ones (F12, S10).
Usage: python -I npm_ages.py <package-lock.json>"""
import datetime
import json
import sys
import time
import urllib.parse
import urllib.request

today = datetime.date.today()
lock = json.load(open(sys.argv[1], encoding="utf-8"))
if lock.get("lockfileVersion") != 3:
    sys.exit(f"expected lockfileVersion 3, found {lock.get('lockfileVersion')}")

packages = []
for path, entry in lock["packages"].items():
    if not path:  # the root entry is this folder itself
        continue
    if not entry.get("resolved", "").startswith("https://registry.npmjs.org/"):
        sys.exit(f"{path} does not come from the npm registry: {entry.get('resolved')}")
    name = path.rsplit("node_modules/", 1)[1]
    packages.append((name, entry["version"]))

young = 0
withdrawn = 0
for name, version in sorted(set(packages)):
    url = "https://registry.npmjs.org/" + urllib.parse.quote(name, safe="@")
    req = urllib.request.Request(url, headers={"User-Agent": "steplet-phase4-age-check (local script)"})
    with urllib.request.urlopen(req, timeout=60) as r:
        doc = json.load(r)
    published = doc.get("time", {}).get(version)
    meta = doc.get("versions", {}).get(version)
    if published is None or meta is None:
        print(f"{name:40} {version:12} NOT IN THE REGISTRY  <-- WITHDRAWN")
        withdrawn += 1
        continue
    day = published[:10]
    age = (today - datetime.date.fromisoformat(day)).days
    deprecated = bool(meta.get("deprecated"))
    flag = ("  <-- under 14 days" if age < 14 else "") + ("  <-- DEPRECATED" if deprecated else "")
    young += age < 14
    withdrawn += deprecated
    print(f"{name:40} {version:12} published {day} {age:5} days old{flag}")
    time.sleep(1)
print(f"{len(set(packages))} npm packages in the lock file; {young} under 14 days old; {withdrawn} withdrawn or deprecated")
