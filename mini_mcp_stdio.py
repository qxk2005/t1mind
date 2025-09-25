#!/usr/bin/env python3
import sys, json

stdin = sys.stdin.buffer
stdout = sys.stdout.buffer

def read_msg():
    buf = bytearray()
    # 读到 header 结束
    while True:
        b = stdin.read(1)
        if not b:
            raise EOFError("stdin closed")
        buf += b
        if buf.endswith(b"\r\n\r\n") or buf.endswith(b"\n\n"):
            break
        if len(buf) > 8192:
            raise RuntimeError("header too large")
    header = buf.decode("latin1")
    length = None
    for line in header.splitlines():
        if line.lower().startswith("content-length:"):
            length = int(line.split(":",1)[1].strip())
            break
    if length is None:
        raise RuntimeError("missing Content-Length")
    body = stdin.read(length)
    if len(body) != length:
        raise EOFError("unexpected EOF")
    return json.loads(body.decode("utf-8"))

def write_msg(obj):
    body = json.dumps(obj, ensure_ascii=False).encode("utf-8")
    header = (
        f"Content-Length: {len(body)}\r\n"
        f"Content-Type: application/json; charset=utf-8\r\n"
        f"\r\n"
    ).encode("latin1")
    stdout.write(header); stdout.write(body); stdout.flush()

TOOLS = [
    {
        "name": "echo",
        "description": "Echo back the provided text.",
        "input": {
            "type": "object",
            "properties": { "text": { "type": "string" } },
            "required": ["text"]
        }
    }
]

def handle(msg):
    mid = msg.get("id")
    method = msg.get("method")
    if method == "initialize":
        write_msg({
            "jsonrpc":"2.0","id":mid,
            "result":{
                "protocolVersion":"2024-05-16",
                "capabilities":{ "tools":{}, "prompts":{}, "resources":{} },
                "serverInfo":{ "name":"mini-stdio", "version":"0.0.1" }
            }
        })
        return
    if method == "tools/list":
        write_msg({
            "jsonrpc":"2.0","id":mid,
            "result": { "tools": TOOLS },
            "server":"mini-stdio"
        })
        return
    if method == "tools/call":
        params = msg.get("params") or {}
        name = params.get("name")
        args = params.get("arguments") or {}
        if name == "echo":
            text = str(args.get("text",""))
            write_msg({
                "jsonrpc":"2.0","id":mid,
                "result": { "content": [ { "type":"text", "text": text } ] }
            })
            return
        write_msg({"jsonrpc":"2.0","id":mid,"error":{"code":-32601,"message":"Unknown tool"}})
        return
    # 忽略通知（如 initialized）
    if mid is not None:
        write_msg({"jsonrpc":"2.0","id":mid,"error":{"code":-32601,"message":"Unknown method"}})

def main():
    while True:
        try:
            msg = read_msg()
            handle(msg)
        except EOFError:
            break
        except Exception as e:
            # 尽量不崩溃，返回通用错误
            try:
                write_msg({"jsonrpc":"2.0","id":None,"error":{"code":-32000,"message":str(e)}})
            except Exception:
                pass

if __name__ == "__main__":
    main()