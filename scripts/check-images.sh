#!/bin/bash

# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

VERSION=0.1.0
RED=$(tput setaf 1)
YELLOW=$(tput setaf 3)
BOLD=$(tput bold)
NORMAL=$(tput sgr0)

usage() { 
    echo 1>&2; 
    echo "📄 list unused images. version $VERSION" 1>&2; 
    echo 1>&2; 
    echo "list all 🌄 images from ${BOLD}${YELLOW}ui/**${NORMAL} direcrories" 1>&2; 
    echo "that was not used in any of *.slint files from ${BOLD}${YELLOW}ui/**${NORMAL} direcrories" 1>&2; 
    
    exit 1; 
}

if ! test -f Cargo.toml; then
    echo "❗Cannot find ${BOLD}${YELLOW}Cargo.toml${NORMAL} in this directory. Should be 🦀 slint project dirrctory" 1>&2; 
    usage;
fi

IN="`find ui/** -name '*.slint'`"
slint_files=(${IN// /;})
if [ ${#slint_files[@]} -eq 0 ]; then
    echo "❗No slint files found." 1>&2; 
    usage;
fi

IN="`find ui/** -name '*.png'`"
image_files=(${IN// /;})
if [ ${#image_files[@]} -eq 0 ]; then
    echo "❗No image files found." 1>&2; 
    usage;
fi
IFS=$'\n' image_files=($(sort <<<"${image_files[*]}")); unset IFS

for image in "${image_files[@]}"; do
    image_name=$(basename ${image})
    # echo $image_name
    in_use=false
    for slint_file in "${slint_files[@]}"; do
        # echo -e "\t$slint_file"
        result=$(grep $image_name $slint_file)
        # echo $result
        if [ ! -z "$result" ]; then
            # echo -e "\t$result"
            in_use=true
            break
        fi
    done

    if test "$in_use" == false; then
        echo "$image"; 
    fi
done
