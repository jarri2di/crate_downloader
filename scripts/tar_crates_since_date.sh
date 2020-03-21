#!/usr/bin/env bash

SINCE_DATE_TIME=${1:-$CURR_MONTH}
CRATES_DIR=$2
DEST_DIR=$3
TARFILE=crates-since-"$SINCE_DATE_TIME".tar

echo -e "\nArchiving new crates since $SINCE_DATE_TIME to $DEST_DIR/$TARFILE ..."

cd "$CRATES_DIR" && \
find . -type f -newermt "$SINCE_DATE_TIME" -exec tar -rf "$DEST_DIR/$TARFILE" {} \;

echo -e "New crates archived to $DEST_DIR/$TARFILE\n"
echo -e "Unpack them like so:"
echo -e "tar -xvf $DEST_DIR/$TARFILE -C /path/to/dest/folder"
