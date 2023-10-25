#!/usr/bin/env bash

# Copies all files generated by the extractor to the necessary locations.
# Run the extractor first (`./gradlew runServer`) before running this.

set -euxo pipefail

cd "$(dirname "$0")"

cp run/valence_extractor_output/{entities,misc}.json ../crates/valence_entity/extracted/
cp run/valence_extractor_output/{blocks,effects,items,packets,sounds}.json ../crates/valence_generated/extracted/
cp run/valence_extractor_output/translation_keys.json ../crates/valence_lang/extracted/
cp run/valence_extractor_output/{registry_codec.dat,tags.json} ../crates/valence_registry/extracted/
cp run/valence_extractor_output/packets.json ../tools/packet_inspector/extracted/
