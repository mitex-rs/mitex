PY := python
BUILD := build.py

.PHONY: develop release wheel wheel-release test clean

develop:
	$(PY) $(BUILD)

release:
	$(PY) $(BUILD) --release

wheel:
	$(PY) $(BUILD) --wheel

wheel-release:
	$(PY) $(BUILD) --wheel --release

test:
	$(PY) $(BUILD) --test

clean:
	cargo clean
	rm -rf target/wheels