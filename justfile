build:
    cargo build --release -p speed-reader

test:
    cargo test

clean:
    cargo clean

install: build
  cargo install

version:
    @grep "^version = " core/Cargo.toml | head -1
    @grep "^version = " gui/Cargo.toml | head -1

# Publish new version (bump core first, then gui)
publish VERSION="0.2.0":
    sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' core/Cargo.toml
    sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' gui/Cargo.toml
    sed -i '' 's/speed-reader-core = ".*"/speed-reader-core = "{{VERSION}}"/' gui/Cargo.toml
    cargo test
    cargo publish -p speed-reader-core
    sleep 5
    @echo "Now publishing speed-reader..."
    cargo publish -p speed-reader
    git add -A
    git commit -m "chore: release v{{VERSION}}"
    git tag v{{VERSION}}
    @echo "Done! Run 'git push --tags' to push."
