#!/usr/bin/env python3
"""Publish encrypted catalogs and write a pinned manifest after every upload succeeds.

Requires boto3, cryptography and the catalog_publish secretspec scope.
Only the six audience-separated catalog files are uploaded, never raw exports.
"""

import base64
import gzip
import hashlib
import json
import os
from pathlib import Path
import uuid
import boto3
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

FILES = (
    "public.sqlite",
    "member.sqlite",
    "public-browser.sqlite",
    "member-browser.sqlite",
    "public-browser.sqlite.gz",
    "member-browser.sqlite.gz",
)


def publish():
    key = base64.b64decode(os.environ["CATALOG_ENCRYPTION_KEY"], validate=True)
    if len(key) != 32:
        raise ValueError("CATALOG_ENCRYPTION_KEY must contain exactly 32 bytes")
    cipher = AESGCM(key)
    client = boto3.client(
        "s3",
        endpoint_url=os.environ["CDN_S3_ENDPOINT"],
        aws_access_key_id=os.environ["CDN_ACCESS_KEY_ID"],
        aws_secret_access_key=os.environ["CDN_SECRET_ACCESS_KEY"],
        region_name="us-east-1",
    )
    release = {"id": str(uuid.uuid4()), "files": {}}
    for name in FILES:
        plain = (Path("data/catalog") / name).read_bytes()
        nonce = os.urandom(12)
        encrypted = nonce + cipher.encrypt(nonce, gzip.compress(plain, mtime=0), None)
        object_key = f"sqlite-releases/{release['id']}/{name}.enc"
        client.put_object(
            Bucket=os.environ["CDN_S3_BUCKET"],
            Key=object_key,
            Body=encrypted,
            ContentType="application/octet-stream",
            CacheControl="public, max-age=31536000, immutable",
        )
        release["files"][name] = {
            "url": os.environ["CDN_PUBLIC_URL"].rstrip("/") + "/" + object_key,
            "sha256": hashlib.sha256(encrypted).hexdigest(),
            "bytes": len(encrypted),
            "plain_sha256": hashlib.sha256(plain).hexdigest(),
            "plain_bytes": len(plain),
        }
        print(f"Published encrypted {name}: {len(encrypted):,} bytes")
    Path("deployments/catalog.json").write_text(json.dumps(release, indent=2) + "\n")
    print("Release manifest ready:", release["id"])


if __name__ == "__main__":
    publish()
