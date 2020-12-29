# ec2-search 
![Test](https://github.com/mocyuto/ec2-search/workflows/Test/badge.svg?branch=master)
![](https://img.shields.io/crates/v/ec2-search)
![](https://img.shields.io/github/v/release/mocyuto/ec2-search?sort=semver)

Search EC2 instance easily

## Installation

### Cargo Install

With Rust's package manager cargo, you can install via:
```shell script
$ cargo install ec2-search
```
If you install the latest master branch commit
```shell script
$ cargo install --git https://github.com/mocyuto/ec2-search --branch master
```

### Homebrew
macOS or Linux

```shell script
$ brew tap mocyuto/ec2-search
$ brew install ec2-search
```

### Setup Completion

If you install by `brew` command, it is set **automatically**.
Otherwise, you run below. 

```zsh
# zsh
$ ec2s completion zsh > /usr/local/share/zsh/site-functions/_ec2s

# bash
$ ec2s completion bash > /usr/local/etc/bash_completion.d/ec2s
```


## Usage

```shell script
$ ec2s help
```

### AWS credentials

ec2-search needs aws credentials, so you need to set credentials.
You can use Environment value or `"~/.aws/credentials"`.

For more information, see [AWS Credentials](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md)

### Instance

Search instance info.
```shell script
$ ec2s instance help
# or alias. see help
$ ec2s i help 
```
#### info
display basic info

```shell script
$ ec2s i info -q api
ID           Name       Status   Type
i-012345678  test-api1  running  t2.micro
i-023456789  test-api2  running  t3.small
counts: 2
```


#### instance ids
display instance ids

```shell script
## like search
$ ec2s instance ids -q "api"
ID           Name
i-012345678  test-api1
i-023456789  test-api2
counts: 2

```

#### instance ips

Display instance public and private IPs.

```shell script
$ ec2s instance ips -q "api"
Private IP   Public IP  Name
10.0.0.1                test-api1
10.0.0.2                test-api2
counts: 2
```

#### instance DNS name

Display instance public and private DNS name

```shell script
$ ec2s i dns -q api
Private DNS                                      Public DNS  Name
 ip-10-10-10-10.ap-northeast-1.compute.internal              test-api1
 ip-10-10-10-11.ap-northeast-1.compute.internal              test-api2
counts: 2
```

### Target Group

```shell script
$ ec2s target-group help
# or alias
$ ec2s tg help
```

#### Info

Display Target Group info

```shell script
$ ec2s tg info -q api
 Name            TargetType  LB
 aaaaa-api       ip          ["aaaaa-bbbb-apialb"]
 bbbbb-api       ip          ["bbbbb-cccc-apiinteralb"]
 api-web         instance    ["prd-api-web"]
counts: 5
```

#### Target Health

Display Target Health

```shell script
$ ec2s tg health -q api-web
 ID                   Port  Status
 i-01002020202000101  80    healthy
counts: 1
```


### Auto Scaling Group

```shell script
$ ec2s auto-scaling-group help
# or alias
$ ec2s asg help
```

#### Info

Display Auto Scaling Group info
```shell script
$ ec2s asg info -q eks
 Name               Instances  Desired  Min  Max
 prd-eks-autoscale  7          7        1    40
 prd-eks-stateful   6          6        4    20
 stg-eks-autoscale  1          1        1    20
 stg-eks-stateful   2          2        1    10
counts: 4
``` 

#### Instances

Display Auto Scaling Group Instances.

```shell script
$ ec2s asg inst -q stg-eks-api-autoscale
 ID                   LifeCycle  InstanceType  AZ               Status
 i-01010101010101011  InService  t3.medium     ap-northeast-1c  Healthy
counts: 1
```
