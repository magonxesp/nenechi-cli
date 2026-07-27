#!/usr/bin/env bash

# release or debug
build_type="$1"
arch="$(dpkg --print-architecture)"
version="$(cat VERSION)"
target_directory="target/$build_type"
bundle_directory="$target_directory/bundle"
deb_package_directory="$bundle_directory/deb"

if [[ -d "$deb_package_directory" ]]; then
  rm -r "$deb_package_directory"
  rm -f "$bundle_directory/*.deb"
fi

mkdir -p "$deb_package_directory/DEBIAN"
mkdir -p "$deb_package_directory/usr/bin"
mkdir -p "$deb_package_directory/etc/nenechi/conf.d"

control=$(cat <<EOF
Package: nenechi
Architecture: $arch
Version: $version
Section: utils
Priority: optional
Maintainer: MagonxESP <janma.360@gmail.com>
Description: Herramientas utiles de nenechi
Homepage: https://github.com/magonxesp/nenechi-cli
EOF
)

conffiles=$(cat <<EOF
/etc/nenechi/config.yaml
EOF
)

echo "$control" > "$deb_package_directory/DEBIAN/control"
echo "$conffiles" > "$deb_package_directory/DEBIAN/conffiles"
cp "$target_directory/nenechi-cli" "$deb_package_directory/usr/bin/nenechi"
cp -r examples/config.yaml "$deb_package_directory/etc/nenechi/config.yaml"

chmod +x "$deb_package_directory/usr/bin/nenechi"

dpkg-deb --build "$deb_package_directory" "$target_directory/bundle/nenechi_${version}_${arch}.deb"
