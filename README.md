# ec2-search [![Build Status](https://travis-ci.com/mocyuto/ec2-search.svg?branch=master)](https://travis-ci.com/mocyuto/ec2-search)

Search EC2 instance easily

## Installation

With Rust's package manager cargo, you can install via:
```sh
$ cargo install ec2-search
```
If you install the latest master branch commit
```sh
$ cargo install --git https://github.com/mocyuto/ec2-search --branch master
```

## Usage

```
$ ec2-search help
$ ec2-search ids -q "*api*"
$ ec2-search ids --exq=front-api
```
