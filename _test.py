import urllib.request, json
base = "http://127.0.0.1:53223/rpc"
data = json.dumps({"method": "detectCodex", "params": {}}).encode()
req = urllib.request.Request(base, data=data, headers={"Content-Type": "application/json"})
resp = urllib.request.urlopen(req, timeout=5)
result = json.loads(resp.read())
info = result.get("data") or result
print("RPC detectCodex test:")
print("  running = " + str(info.get("running")))
print("  pid = " + str(info.get("pid")))
print("  method = " + str(info.get("method")))
if info.get("running"):
    print("  STATUS: GUI SHOULD SHOW RUNNING (Codex detected)")
else:
    print("  STATUS: GUI WOULD SHOW STOPPED")
