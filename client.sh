#!/bin/sh

export ENV_LISTEN_ADDR=127.0.0.1:51804

keys(){
  jq -c -n '{
    max_num_keys: 42,
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/GetKeys
}

set_pair(){

  jq -c -n '{
    key: "helo",
    val: "wrld",
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Set

  key=$( printf hh | base64 )
  val=$( printf ww | base64 )

  jq -c -n --arg key "$key" --arg val "$val" '{
    key: $key,
    val: $val,
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Set

}

get(){
  jq -c -n '{
    key: "helo",
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Get
}

exists(){
  jq -c -n '{ key: "helo" }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Exists
}

mget(){
  jq -c -n --arg val $(printf hh | base64) '{
    keys: [
      "helo",
      $val,
      "invldkey"
    ],
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/MultiGet
}

del(){
  jq -c -n --arg val $(printf hh | base64) '{
    key: "helo",
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Del

  jq -c -n --arg val $(printf hh | base64) '{
    key: "HELO",
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Del
}

mset(){
  jq -c -n --arg val $(printf hh | base64) '{
    pairs: [
      {
        "key": "helo",
        "val": "wrld",
      },
      {
        "key": "helo",
        "val": "wrld",
      }
    ],
  }' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/MultiSet
}

count(){
  jq -c -n '{}' |
  grpcurl \
    -plaintext \
    -import-path ./proto \
    -proto minkvs.proto \
    -d @ \
    "${ENV_LISTEN_ADDR}" \
    minkvs.v1.MinKvsService/Count
}

desc(){
  grpcurl \
    -plaintext \
    "${ENV_LISTEN_ADDR}" \
    list

  grpcurl \
    -plaintext \
    "${ENV_LISTEN_ADDR}" \
    list minkvs.v1.MinKvsService

  grpcurl \
    -plaintext \
    "${ENV_LISTEN_ADDR}" \
    list minkvs.v1.MinKvsService
}

desc

set_pair | jq -c
keys | jq -c
get | jq -c
exists | jq -c
mget | jq
del | jq -c
mset | jq -c
count | jq -c
