import json
import base64
import hashlib

def hash(block):
  encoded = json.dumps((block['predecessor'], block['transactions'],
    block['difficulty'], block['nonce']),
    sort_keys=True, separators=(',',':'))
  print(encoded)
  print(hashlib.sha256(encoded.encode('utf-8')).digest())
  return '0x' + str(base64.b16encode(hashlib.sha256(encoded.encode('utf-8')).digest())).lower()


block = {"init":{"predecessor":"","nonce":0, "difficulty":0, "transactions":[ {"inputs":[],"outputs":[{"id":73,"amount":30}]}], "hash":"0xdcb3d5ee85f43e20e5844b787738941cc780eaac8200cb6734ca13cb4f8d1f85"}}
block = block["init"]

print(hash(block))