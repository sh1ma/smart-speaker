import socket
import re
import subprocess
import requests
import os

pattern = re.compile('WORD="([^"]+)"')

kotonoha_host = os.environ.get("KOTONOHA_HOST", "localhost")
kotonoha_port = os.environ.get("KOTONOHA_PORT", "8081")
WAKE_WORD = os.environ.get("WAKE_WORD", "ハロー")

host = "locahost"
port = 10500


s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect((host, port))

url = f"http://{kotonoha_host}:{kotonoha_port}"


def synthesis(text):
    data = {
        "speaker": "3",
        "text": text,
    }
    audio_query = requests.post(
        f"http://{kotonoha_host}:50021/audio_query", params=data
    ).json()

    print(audio_query)
    audio_query["speedScale"] = 1.3

    response = requests.post(
        f"http://{kotonoha_host}:50021/synthesis?speaker=3", json=audio_query
    )
    # print(response.content)
    return response.content


while True:
    data = s.recv(1024)
    if found_word := pattern.search(data.decode("utf-8")):
        print(found_word.group(1))
        if found_word.group(1) == "うさみ":
            result = subprocess.run(["adinrec", "out.wav"])
            with open("out.wav", "rb") as f:
                response = requests.post(
                    f"{url}/transcribe",
                    data=f,
                )
                response
                talked = requests.post(
                    f"{url}/talk",
                    json={"text": response.json()["text"]},
                ).json()
                resp_wav = synthesis(talked["output"])

                with open("resp.wav", "wb") as f:
                    f.write(resp_wav)
            subprocess.run(["aplay", "-D", "plughw:2,0", "-"], input=resp_wav)
