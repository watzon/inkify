#!/usr/bin/env bash

all_nerd_fonts=(
    "3270"
    "Agave"
    "AnonymousPro"
    "Arimo"
    "AurulentSansMono"
    "BigBlueTerminal"
    "BitstreamVeraSansMono"
    "CascadiaCode"
    "CodeNewRoman"
    "ComicShannsMono"
    "Cousine"
    "DaddyTimeMono"
    "DejaVuSansMono"
    "DroidSansMono"
    "EnvyCodeR"
    "FantasqueSansMono"
    "FiraCode"
    "FiraMono"
    "Go-Mono"
    "Gohu"
    "Hack"
    "Hasklig"
    "HeavyData"
    "Hermit"
    "iA-Writer"
    "IBMPlexMono"
    "Inconsolata"
    "InconsolataGo"
    "InconsolataLGC"
    "IntelOneMono"
    "Iosevka"
    "IosevkaTerm"
    "JetBrainsMono"
    "Lekton"
    "LiberationMono"
    "Lilex"
    "Meslo"
    "Monofur"
    "Monoid"
    "Mononoki"
    "MPlus"
    "NerdFontsSymbolsOnly"
    "Noto"
    "OpenDyslexic"
    "Overpass"
    "ProFont"
    "ProggyClean"
    "RobotoMono"
    "ShareTechMono"
    "SourceCodePro"
    "SpaceMono"
    "Terminus"
    "Tinos"
    "Ubuntu"
    "UbuntuMono"
    "VictorMono"
)

mkdir -p ./nerd_fonts

# Download each font, un-tar it, and install it
for font in "${all_nerd_fonts[@]}"; do
    echo "Downloading $font..."
    wget "https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.2/$font.tar.xz"
    
    mkdir -p "./$font"
    tar -xf "./$font.tar.xz" -C "./$font"
    rm "$font.tar.xz"

    # Remove fonts contining "NerdFontMono" and "NerdFontPropo" in the name
    rm "./$font/"*NerdFontMono*
    rm "./$font/"*NerdFontProp*

    # Move the font directory to the nerd_fonts directory
    mv "./$font" ./nerd_fonts
done