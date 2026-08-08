#!/bin/sh

dname=$( dirname "$0" )
cd $dname

buf lint
buf format --write
