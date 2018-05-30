# Zigbee2MQTT Hass.io Updater

This is a small web service, written in Rust, that provides a method for building new Docker images for the [zigbee2mqtt Hassio add-on]() upon push events in the [zigbee2mqtt](https://github.com/Koenkk/zigbee2mqtt) library via a webhook.

##### Motivation

`zigbee2mqtt` is growing quickly and support for new devices is being added frequently. The library does not use versioned releases. As a result, the Hassio add-on just checks out the `zigbee2mqtt` master branch from source control. When changes are made to the underlying library, the add on is not aware and new Docker images are not built. This integration will allow new Docker images to be built and pushed to Docker Hub with each update of `zigbee2mqtt`.
