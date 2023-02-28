# hhm-cli

spin up and down servers because it's funny

## example config

```toml
prefix = "hhm-cli"
api_key = "no"
ssh_keys = ["main"]

image = "debian-11"
instance_type = "cpx11"
zone = "ash"

cloud_init = """#cloud-config

runcmd:
  - echo "hello, %HHM_ID%"
"""

[labels]
spun_with = "hhm-cli"
```
