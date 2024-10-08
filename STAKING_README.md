# Staking

## eth
目前仅支持eth-mainnet, holesky-testnet这两个链的质押

### 新建key

`--withdrawal-credentials`: 提取地址,可以指定也可以不写,自动派生

```
./eth-staking-smith new-mnemonic \
    --keystore_password 11111111 \
    --num_validators 1 \
    --chain holesky \
    --withdrawal-credentials 0x28B9FEAE1f3d76565AAdec86E7401E815377D9Cc
```

### 新建key之后进行deposit

`--from-path`: 指定私钥的文件路径  
`--staking-rpc`: 需要质押到的网络的rpc

```
./eth-staking-smith new-mnemonic \
    --keystore_password 11111111 \
    --num_validators 1 \
    --chain holesky \
    --withdrawal-credentials 0x28B9FEAE1f3d76565AAdec86E7401E815377D9Cc \
    --from-path ./sk \
    --staking-rpc https://ethereum-holesky-rpc.publicnode.com
```

### 导入助记词之后进行deposit

```
./eth-staking-smith existing-mnemonic \
  --mnemonic "palace parade smoke alert thought ship luggage crouch during shrug budget height fan author ask wear catch gaze half girl song tunnel fossil wasp" \
  --keystore_password 11111111 \
  --num_validators 1 \
  --chain holesky \
  --withdrawal-credentials 0x28B9FEAE1f3d76565AAdec86E7401E815377D9Cc \
  --from-path ./sk \
  --staking-rpc https://ethereum-holesky-rpc.publicnode.com
```

## 自定义网络

### 以后会添加的功能  
- [ ] 输入自定义合约地址
- [ ] 输入自定义交易的data hex
- [ ] 等等
