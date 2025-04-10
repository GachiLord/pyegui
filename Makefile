build:
	rm -rf target/wheels/*
	docker run --rm -v $(PWD):/io ghcr.io/pyo3/maturin build --release --sdist
	.venv/bin/maturin build --release --target x86_64-pc-windows-gnu

upload:
	.venv/bin/maturin upload target/wheels/*

upload-test:
	python3 -m twine upload --repository testpypi target/wheels/*

debug:
	.venv/bin/python debug.py

develop:
	.venv/bin/maturin develop

venv:
	python3 -m venv .venv
	.venv/bin/pip install maturin

