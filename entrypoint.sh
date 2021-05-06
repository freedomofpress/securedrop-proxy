#!/bin/sh

cd /home/user/projects/securedrop-proxy
virtualenv .venv
source .venv/bin/activate
pip install --require-hashes -r requirements/requirements.txt
pip install --require-hashes -r requirements/dev-requirements.txt
./sd-proxy.py ./config.yaml
