# Staking

## eth
目前仅支持eth-mainnet, holesky-testnet这两个链的质押

### 新建key

`--withdrawal-credentials`: 提取地址,可以指定也可以不写,自动派生  
`--num_validators`: 根据一个助记词生成X个validator信息,密码都是相同的 `--keystore_password`  
`--output`: 输出validator相关信息的文件夹路径,如下所示的结构,默认输出路径`./`   
```
cd ./validators
-rw-rw-r-- 1 cloud cloud 2120 10月 12 16:16 export.json
drwxrwxr-x 2 cloud cloud 4096 10月 12 16:16 validator-0/

cd validator-0/
-rw-rw-r-- 1 cloud cloud  724 10月 12 16:16 deposit-data.json
-rw-rw-r-- 1 cloud cloud  167 10月 12 16:16 mnemonic.json
-rw-rw-r-- 1 cloud cloud   64 10月 12 16:16 private-key
-rw-rw-r-- 1 cloud cloud  890 10月 12 16:16 voting-keystore.json
```

- 使用cmd指令
```
./eth-staking-smith new-mnemonic \
    --keystore_password 11111111 \
    --num_validators 1 \
    --chain holesky \
    --withdrawal-credentials 0x28B9FEAE1f3d76565AAdec86E7401E815377D9Cc
    --output ./
```

- 使用环境变量

在配置文件中设置

```
export new_mnemonic_config="./eth-staking-smit/config/new_mnemonic.toml"

./eth-staking-smith new-mnemonic

```

### 导入key(同新建key差不多)

### 质押

- 使用cmd
```
./eth-staking-smith staking \
    --chain holesky \
    --rpc "https://ethereum-holesky-rpc.publicnode.com" \
    --from_path "./sk" \
    --staking_address "0x28B9FEAE1f3d76565AAdec86E7401E815377D9Cc" \
    --export_path "./validators/export.json"
```

- 使用环境变量
```
export staking_config="./eth-staking-smit/config/staking.toml"

./eth-staking-smith staking
```

## 自定义网络

默认自定义网络使用的合约和eth的标准质押合约是一样的, 如果不一样则无法使用

新增字段`--staking-address`, 配合`chain == custom`的时候使用

## 配合lighthouse使用
1. 使用`new-mnemonic`之后,会输出对应信息到指定文件夹`./validators/`
2. 使用lighthouse导入 `./lighthouse --network holesky account validator import --directory ./validators`
3. 输入密码就是使用`new-mnemonic`指定的`--keystore_password`

