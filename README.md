# ec2-search ![Test](https://github.com/mocyuto/ec2-search/workflows/Test/badge.svg?branch=master)

Search EC2 instance easily

## Installation

### Cargo Install

With Rust's package manager cargo, you can install via:
```sh
$ cargo install ec2-search
```
If you install the latest master branch commit
```sh
$ cargo install --git https://github.com/mocyuto/ec2-search --branch master
```

### Homebrew
macOS or Linux

```sh
$ brew tap mocyuto/ec2-search
$ brew install ec2-search
```

## Usage

```sh
$ ec2s help
```

### AWS credentials

ec2-search needs aws credentials, so you need to set credentials.
You can use Environment value or `"~/.aws/credentials"`.

For more information, see [AWS Credentials](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md)

### Instance-ids

Display instance ids

```sh
## like search
$ ec2s ids -q "api"
i-012345678 : test-api1
i-023456789 : test-api2
counts: 2

## search exact query match
$ ec2s ids --exq=front-api

## search with ids
$ ec2s ids --ids i-abcde12345
```

### Instance-private-ips

Display instance private IPs.

```sh
$ ec2s ips -q "api"
["10.0.0.1"] : test-api1
["10.0.0.2"] : test-api2
counts: 2
```
