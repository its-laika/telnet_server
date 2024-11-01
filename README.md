# telnet_server
_...because nothing beats the charm of the seventies!_

## Goals
This lib should provide everything to build a service (based on telnet) without
having to touch any telnet or tcp specifics. Message communication should only
happen between this lib and the code providing the service. 

## Status
Non functional and heavily WIP. **DO NOT USE!**

## Running telnet
As telnet is not anymore part of modern operating systems (thank god), I've
created a minimal Dockerfile that let's me use telnet on CLI. Telnet starts via
this command:

```sh
./telnet/run_telnet.sh HOST PORT
```

For development, _HOST_ is "host.docker.internal" and _PORT_ is 9000.

## For f*cks sake, why telnet???
Because it looked interesting. Honestly, even if I get it finished, the library
likely won't be used _at all_.

## License
The code is licensed under the [MIT License](LICENSE). The RFCs in the
[resources](resources) folder have a different copyright but are allowed (and
encouraged) to be copied and redistributed if unchanged.