build:
	rm -rf target/wheels/*
	.venv/bin/maturin build --release --target x86_64-pc-windows-gnu
	docker run --rm -v $(PWD):/io ghcr.io/pyo3/maturin build --release --sdist
build-manylinux:
	docker run --entrypoint cargo --rm -v $(PWD):/io ghcr.io/pyo3/maturin build --release 

upload:
	python3 -m twine upload upload target/wheels/*

upload-test:
	python3 -m twine upload --repository testpypi target/wheels/*

python:
	.venv/bin/python

debug:
	.venv/bin/python debug.py

develop:
	.venv/bin/maturin develop

venv:
	python3 -m venv .venv
	.venv/bin/pip install maturin

