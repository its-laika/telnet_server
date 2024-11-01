# telnet_server
_...because nothing beats the charm of the seventies!_

## Goals
This lib should provide everything to build a service (based on TELNET) without
having to touch any TELNET or tcp specifics. Message communication should only
happen between this lib and the code providing the service. 

See [src/bin/main.rs](src/bin/main.rs#l23) on how I think this should work.

## Status
Non functional and heavily WIP. **DO NOT USE!**

## Running TELNET
As TELNET is not anymore part of modern operating systems (thank god), I've
created a minimal Dockerfile that let's me use TELNET on CLI. TELNET starts via
this command:

```sh
./telnet/run_telnet.sh HOST PORT
```

For development, _HOST_ is "host.docker.internal" and _PORT_ is 9000.

## For f*cks sake, why TELNET???
Because it looked interesting. Honestly, even if I get it finished, the library
likely won't be used _at all_.

## License
The code is licensed under the [MIT License](LICENSE). The RFCs in the
[resources](resources) folder have a different copyright but are allowed (and
encouraged) to be copied and redistributed if unchanged.