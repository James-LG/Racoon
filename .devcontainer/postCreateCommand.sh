#!/bin/bash

/usr/bin/python3 -m venv /workspaces/Skyscraper/.env
/workspaces/Skyscraper/.env/bin/pip install -r tests/lxml_tests/requirements.txt
echo "source /workspaces/Skyscraper/.env/bin/activate" >> ~/.bashrc
