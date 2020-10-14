# ec2-search [![Build Status](https://travis-ci.com/mocyuto/ec2-search.svg?branch=master)](https://travis-ci.com/mocyuto/ec2-search)

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

```sh
$ brew tap mocyuto/ec2-search
$ brew install ec2-search
```

## Usage

```sh
$ ec2-search help
```

### instance-ids

display instance ids

```
## like search
$ ec2-search ids -q "api"
i-012345678 : test-api1
i-023456789 : test-api2
counts: 2

## search exact query match
$ ec2-search ids --exq=front-api
```

### instance-private-ips

display instance private ips.

```sh
$ ec2-search ips -q "api"
["10.0.0.1"] : test-api1
["10.0.0.2"] : test-api2
counts: 2
```
