#!/bin/bash

# Ensure that curl is installed
if ! command -v curl >/dev/null 2>&1; then
  echo 'Error: curl is not installed.' >&2
  exit 1
fi

GIT_ROOT=$(git rev-parse --show-toplevel)
DEST_DIR="${GIT_ROOT}/data/GameMaps"

download_file() {
  local url=$1
  local dest=$2
  curl -L -o "$dest" "$url"
}

download_file_if_not_exists() {
  local url=$1
  local dest=$2
  if [[ ! -f "$dest" ]]; then
    download_file "$url" "$dest"
  fi
}

make_directory() {
  local dir=$1
  mkdir -p "$dir"
}

files=(
  # Scene files
  "Scene/bridgeA-1.scene"
  "Scene/bridgeA.scene"
  "Scene/bridgeB-L.scene"
  "Scene/bridgeB-R.scene"
  "Scene/cbridge08.scene"
  "Scene/cbridge09.scene"
  "Scene/cbridge14.scene"
  "Scene/ch-tree01.scene"
  "Scene/ch-tree02.scene"
  "Scene/ch-tree03.scene"
  "Scene/ch-tree04.scene"
  "Scene/ch-tree05.scene"
  "Scene/ch-tree06.scene"
  "Scene/cockhorsea.scene"
  "Scene/cockhorseb.scene"
  "Scene/cockhorsec.scene"
  "Scene/cockhorsed.scene"
  "Scene/cushaw01.scene"
  "Scene/cushaw01a.scene"
  "Scene/cushaw02.scene"
  "Scene/cushaw02a.scene"
  "Scene/cushaw03.scene"
  "Scene/cushaw03a.scene"
  "Scene/d_map0200.scene"
  "Scene/d_map0201.scene"
  "Scene/d_map0202.scene"
  "Scene/d_map0203.scene"
  "Scene/d_map0204.scene"
  "Scene/d_map0205.scene"
  "Scene/d_map0206.scene"
  "Scene/d_map0207.scene"
  "Scene/d_map0208.scene"
  "Scene/d_map0210.scene"
  "Scene/d_map0211.scene"
  "Scene/d_map0212.scene"
  "Scene/d_map0213.scene"
  "Scene/d_map0214.scene"
  "Scene/d_map0215.scene"
  "Scene/d_map0216.scene"
  "Scene/d_map0217.scene"
  "Scene/d_map0218.scene"
  "Scene/d_map0219.scene"
  "Scene/d_map0220.scene"
  "Scene/d_map0221.scene"
  "Scene/faction01.scene"
  "Scene/faction02.scene"
  "Scene/faction03.scene"
  "Scene/furni01.scene"
  "Scene/furni02.scene"
  "Scene/furni03.scene"
  "Scene/furni04.scene"
  "Scene/furni05.scene"
  "Scene/furni06.scene"
  "Scene/furni07.scene"
  "Scene/furni08.scene"
  "Scene/furni09.scene"
  "Scene/furni10.scene"
  "Scene/furni11.scene"
  "Scene/furni12.scene"
  "Scene/furni13.scene"
  "Scene/furni14.scene"
  "Scene/furni15.scene"
  "Scene/furni16.scene"
  "Scene/furni17.scene"
  "Scene/furni18.scene"
  "Scene/furni19.scene"
  "Scene/furni20.scene"
  "Scene/furni21.scene"
  "Scene/furnishings"
  "Scene/house13.scene"
  "Scene/house14.scene"
  "Scene/house15.scene"
  "Scene/house16.scene"
  "Scene/house17.scene"
  "Scene/house18.scene"
  "Scene/house19.scene"
  "Scene/house20.scene"
  "Scene/house21.scene"
  "Scene/house22.scene"
  "Scene/house23.scene"
  "Scene/house24.scene"
  "Scene/hq01.scene"
  "Scene/hq02.scene"
  "Scene/hq07.scene"
  "Scene/hq08.scene"
  "Scene/hq09.scene"
  "Scene/lei01.scene"
  "Scene/lei02.scene"
  "Scene/lei03.scene"
  "Scene/lei04.scene"
  "Scene/nh03-01.scene"
  "Scene/nh03-02.scene"
  "Scene/nh03-03.scene"
  "Scene/nh04-01.scene"
  "Scene/nh04-02.scene"
  "Scene/nh04-03.scene"
  "Scene/nh04-04.scene"
  "Scene/nh04-05.scene"
  "Scene/nh04-06.scene"
  "Scene/pai.scene"
  "Scene/pearla.scene"
  "Scene/pearlb.scene"
  "Scene/pearlc.scene"
  "Scene/pearld.scene"
  "Scene/pole.scene"
  "Scene/sand01.scene"
  "Scene/sand02.scene"
  "Scene/sand03.scene"
  "Scene/sand04.scene"
  "Scene/sand05.scene"
  "Scene/sand06.scene"
  "Scene/sand07.scene"
  "Scene/sand08.scene"
  "Scene/sand11.scene"
  "Scene/sand12.scene"
  "Scene/sand13.scene"
  "Scene/sand14.scene"
  "Scene/sand15.scene"
  "Scene/sand16.scene"
  "Scene/sand17.scene"
  "Scene/sand18.scene"
  "Scene/sand20.scene"
  "Scene/sand21.scene"
  "Scene/sand22.scene"
  "Scene/sand23.scene"
  "Scene/sand24.scene"
  "Scene/sand25.scene"
  "Scene/sand26.scene"
  "Scene/sand27.scene"
  "Scene/sand28.scene"
  "Scene/sand29.scene"
  "Scene/sand30.scene"
  "Scene/sand31.scene"
  "Scene/sand32.scene"
  "Scene/sand33.scene"
  "Scene/sand34.scene"
  "Scene/sand35.scene"
  "Scene/sand36.scene"
  "Scene/snowman0.scene"
  "Scene/snowman1.scene"
  "Scene/stand01.scene"
  "Scene/stand02.scene"
  "Scene/stand03.scene"
  "Scene/stand04.scene"
  "Scene/stand05.scene"
  "Scene/stand06.scene"
  "Scene/stand07.scene"
  "Scene/stand08.scene"
  "Scene/stand09.scene"
  "Scene/stand10.scene"
  "Scene/swinga.scene"
  "Scene/swingb.scene"
  "Scene/tree01.scene"
  "Scene/wall-l1.scene"
  "Scene/wall-l2.scene"
  "Scene/wall-r1.scene"
  "Scene/wall-r2.scene"
  "Scene/wbridge.scene"
  "Scene/wbridge1.scene"
  "Scene/windmilla.scene"
  "Scene/windmillb.scene"
  # Scene Part
  "ScenePart/bridge01.Part"
  "ScenePart/bridge02.Part"
  "ScenePart/bridge03.Part"
  "ScenePart/bridge05-1.Part"
  "ScenePart/bridge05-2.Part"
  "ScenePart/bridge05.Part"
  "ScenePart/bridge06-0.Part"
  "ScenePart/bridge06-1.Part"
  "ScenePart/bridge06-2.Part"
  "ScenePart/bridge06-3.Part"
  "ScenePart/bridge06-4.Part"
  "ScenePart/bridge06-5.Part"
  "ScenePart/bridge06-6.Part"
  "ScenePart/bridge06-7.Part"
  "ScenePart/bridge06-8.Part"
  "ScenePart/bridge07.Part"
  "ScenePart/bridgeA-1.scene"
  "ScenePart/cbridge01.Part"
  "ScenePart/cbridge02-1-saya.Part"
  "ScenePart/cbridge02-1.Part"
  "ScenePart/cbridge02-2-saya.Part"
  "ScenePart/cbridge02-3-saya.Part"
  "ScenePart/cbridge02-4-saya.Part"
  "ScenePart/cbridge02-5-saya.Part"
  "ScenePart/cbridge02-6-saya.Part"
  "ScenePart/cbridge02-7-saya.Part"
  "ScenePart/cbridge02-8-saya.Part"
  "ScenePart/cbridge03-1-saya.Part"
  "ScenePart/cbridge03-2-saya.Part"
  "ScenePart/chair-D.Part"
  "ScenePart/chair-L.Part"
  "ScenePart/chair-R.Part"
  "ScenePart/chair-U.Part"
  "ScenePart/cockpit01.Part"
  "ScenePart/cockpit02.Part"
  "ScenePart/cockpit03.Part"
  "ScenePart/cockpit04.Part"
  "ScenePart/cockpit05.Part"
  "ScenePart/cockpit06.Part"
  "ScenePart/cockpit07.Part"
  "ScenePart/cockpit08.Part"
  "ScenePart/d_map0200.Part"
  "ScenePart/d_map0201.Part"
  "ScenePart/d_map0202.Part"
  "ScenePart/d_map0203.Part"
  "ScenePart/d_map0204.Part"
  "ScenePart/d_map0205.Part"
  "ScenePart/d_map0206.Part"
  "ScenePart/d_map0207.Part"
  "ScenePart/d_map0208.Part"
  "ScenePart/d_map0210.Part"
  "ScenePart/d_map0211.Part"
  "ScenePart/d_map0212.Part"
  "ScenePart/d_map0213.Part"
  "ScenePart/d_map0214.Part"
  "ScenePart/d_map0215.Part"
  "ScenePart/d_map0216.Part"
  "ScenePart/d_map0217.Part"
  "ScenePart/d_map0218.Part"
  "ScenePart/d_map0219.Part"
  "ScenePart/d_map0220.Part"
  "ScenePart/d_map0221.Part"
  "ScenePart/faction01.Part"
  "ScenePart/faction02.Part"
  "ScenePart/faction03.Part"
  "ScenePart/pai.Part"
  "ScenePart/sand01.Part"
  "ScenePart/sand02.Part"
  "ScenePart/sand03.Part"
  "ScenePart/sand04.Part"
  "ScenePart/sand05.Part"
  "ScenePart/sand06.Part"
  "ScenePart/sand07.Part"
  "ScenePart/sand08.Part"
  "ScenePart/sand11.Part"
  "ScenePart/sand12.Part"
  "ScenePart/sand13.Part"
  "ScenePart/sand14.Part"
  "ScenePart/sand15.Part"
  "ScenePart/sand16.Part"
  "ScenePart/sand17.Part"
  "ScenePart/sand18.Part"
  "ScenePart/sand19.Part"
  "ScenePart/sand20.Part"
  "ScenePart/sand21.Part"
  "ScenePart/sand22.Part"
  "ScenePart/sand23.Part"
  "ScenePart/sand24.Part"
  "ScenePart/sand25.Part"
  "ScenePart/sand26.Part"
  "ScenePart/sand27.Part"
  "ScenePart/sand28.Part"
  "ScenePart/sand29.Part"
  "ScenePart/sand30.Part"
  "ScenePart/sand31.Part"
  "ScenePart/sand32.Part"
  "ScenePart/sand33.Part"
  "ScenePart/sand34.Part"
  "ScenePart/sand35.Part"
  "ScenePart/sand36.Part"
  "ScenePart/stand01.Part"
  "ScenePart/stand02.Part"
  "ScenePart/stand03.Part"
  "ScenePart/stand04.Part"
  "ScenePart/stand05.Part"
  "ScenePart/stand06.Part"
  "ScenePart/stand07.Part"
  "ScenePart/stand08.Part"
  "ScenePart/stand09.Part"
  "ScenePart/stand10.Part"
  "ScenePart/stone01.Part"
  "ScenePart/stone02.Part"
  "ScenePart/stone03.Part"
  "ScenePart/stone04.Part"
  "ScenePart/stone05.Part"
  "ScenePart/stone06.Part"
  "ScenePart/table.Part"
  "ScenePart/tree01.Part"
  "ScenePart/wbridge01.Part"
  "ScenePart/wbridge02-1.Part"
  "ScenePart/wbridge02.Part"
  "ScenePart/wbridge03-1.Part"
  "ScenePart/wbridge03-2.Part"
  "ScenePart/wbridge03-3.Part"
  "ScenePart/wbridge03-4.Part"
  "ScenePart/wbridge03-5.Part"
  "ScenePart/wbridge03.Part"
  # Maps
  "map/arena.DMap"
  "map/newbie.DMap"
  "map/newcanyon.DMap"
)

for file in "${files[@]}"; do
  dest="${DEST_DIR}/${file}"
  url="${S3_ENDPOINT}/GameMaps/${file}"
  make_directory "$(dirname "$dest")"
  echo "Downloading ${url} to ${dest}"
  download_file_if_not_exists "$url" "$dest"
done
