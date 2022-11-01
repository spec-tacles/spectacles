# Spectacles

Spectacles provides a pluggable interface for routing data between Discord application services.

## Overview

Services communicate using brokers. Each broker implements a specific transportation protocol, e.g.
HTTP, MQTT, or AMQP. These brokers are applications that communicate locally on STDIN/OUT. To
communicate remotely, applications can pipe bson data into or out of a broker.

## Example

`gateway | http` -> ✨network✨ -> `http | bot`

To ease implementation, a JSON "broker" is provided that simply translates BSON data into JSON.
