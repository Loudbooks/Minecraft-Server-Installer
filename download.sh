#!/bin/sh

arm="false"
os=""

if [ "$(uname -m)" == 'arm64' ]; then
    arm="true"
fi

case "$(uname -sr)" in
   Darwin*)
     os="macOS"
     ;;
     
   Linux*)
     os="linux"
     ;;

   CYGWIN*|MINGW*|MINGW32*|MSYS*)
     os="windows"
     ;;
   *)
     echo Unknown OS. Unable to continue.
     exit
     ;;
esac

echo
echo Operating system is set to $os. arm=$arm

if test -f ./server.jar; then
    echo server.jar already exists. Make sure this folder is empty before running!
    exit
fi

if ! test -f ./java/bin/java; then
    echo "Java does not exist. Downloading!"
    if [ "$arm" ]; then
        echo "Detected arm system."
    
        if [ "$os" == "macOS" ]; then
            curl "https://download.oracle.com/java/17/latest/jdk-17_macOS-aarch64_bin.tar.gz" --output java.tar.gz
        elif [ "$os" == "linux" ]; then
            curl "https://download.oracle.com/java/17/latest/jdk-17_linux-aarch64_bin.tar.gz" --output java.tar.gz
        fi
    else
        echo "Detected non x64 system."
    
        if [ "$os" == "macOS" ]; then
            curl "https://download.oracle.com/java/17/latest/jdk-17_macos-x64_bin.tar.gz" --output java.tar.gz
        elif [ "$os" == "linux" ]; then
            curl "https://download.oracle.com/java/17/latest/jdk-17_linux-x64_bin.tar.gz" --output java.tar.gz
        elif [ "$os" == "windows" ]; then
            curl "https://download.oracle.com/java/17/latest/jdk-17_windows-x64_bin.zip" --output java.tar.gz
        fi
    fi

    tar xvzf java.tar.gz
    rm java.tar.gz
    mv jdk-17.0.10 java

else
    echo "Java is already downloaded."
fi

echo
echo "What kind of server are you looking to host? Select a number from below."
echo "1: Vanilla - No plugins or mods."
echo "2: Paper - Plugins will be supported."
echo "3: Fabic - Fabric mods will be supported."
echo
#echo "4: Forge - Forge mods will be supported." // Forge does not have a server jar download. Smh.

read -p "Enter your selection: (1-4) " type

while ! [[ "$type" =~ ^[0-9]+$ ]] || [[ "$type" -lt 1 || "$type" -gt 4 ]]; do
    echo "Invalid selection. Try again."
    read -p "Enter your selection: (1-4) " type
done
    
if ! test -d ./libs; then 
    echo Downloading libs...
    mkdir libs

    if [ "$arm" ]; then
        echo "Detected arm system."

        if [ "$os" == "macOS" ]; then
            curl -L "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-macos-arm64" --output-dir libs --output jq
        elif [ "$os" == "linux" ]; then
            curl -L "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-linux-arm64" --output-dir libs --output jq
        fi
    else
        echo "Detected x64 system."

        if [ "$os" == "macOS" ]; then
            curl -L "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-macos-amd64" --output-dir libs --output jq
        elif [ "$os" == "linux" ]; then
            curl -L "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-linux-amd64" --output-dir libs --output jq
        elif [ "$os" == "windows" ]; then
            curl -L "https://github.com/jqlang/jq/releases/download/jq-1.7.1/jq-windows-amd64.exe" --output-dir libs --output jq
        fi
    fi

    chmod +rwx ./libs/jq
fi

jq_usage="./libs/jq"

if [ "$os" == "windows" ]; then
    jq_usage="./libs/jq.exe"
fi

case $type in
    1)        
        latest_version=$(curl -s "https://launchermeta.mojang.com/mc/game/version_manifest.json" | $jq_usage -r '.versions | map(select(.type == "release"))[0].id')
        echo Using Minecraft version $latest_version
        
        echo Downloading server jar...
        curl server.jar $($jq_usage -r ".downloads.server.url" <<< $(curl -s $(curl -s https://launchermeta.mojang.com/mc/game/version_manifest.json | $jq_usage -r ".versions[1].url"))) --output server.jar
        
        echo Successfully downloaded vanilla server jar.
        ;;
    2)  
        paper_version=$(curl -s https://api.papermc.io/v2/projects/paper | \
            $jq_usage -r '.versions[-1]')
        
        latest_build=$(curl -s https://api.papermc.io/v2/projects/paper/versions/${paper_version}/builds | \
            $jq_usage -r '.builds | map(select(.channel == "default") | .build) | .[-1]')
        
        echo Using Paper version $paper_version and build $latest_build
        
        if [ -z "$latest_build" ]; then
            echo FATAL: Paper version not found.
            exit
        fi
        
        jar_name=paper-${paper_version}-${latest_build}.jar
        
        url="https://api.papermc.io/v2/projects/paper/versions/${paper_version}/builds/${latest_build}/downloads/${jar_name}"
        
        curl $url --output server.jar
        echo Successfully downloaded Paper server jar.
        ;;
    3)
        game_version=$(curl -s "https://meta.fabricmc.net/v2/versions" | $jq_usage -r '.game | map(select(.stable == true))[0].version')
        fabric_version=$(curl -s "https://meta.fabricmc.net/v2/versions/loader" | $jq_usage -r 'map(select(.stable == true))[0].version')
        
        echo Using game version $game_version and fabric version $fabric_version 
        
        curl https://meta.fabricmc.net/v2/versions/loader/${game_version}/${fabric_version}/1.0.0/server/jar --output server.jar
        
        echo Successfully downloaded Fabric server jar.
        ;;
esac
