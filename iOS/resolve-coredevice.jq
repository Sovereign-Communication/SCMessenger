[
  .result.devices[]
  | select(
      (.identifier // "") != ""
      and (.hardwareProperties.udid // "") != ""
      and (.connectionProperties.pairingState // "") == "paired"
      and (.hardwareProperties.platform // "") == "iOS"
      and (.hardwareProperties.reality // "") == "physical"
      and (
        .identifier == $id
        or (.hardwareProperties.udid // "") == $id
      )
    )
  | {
      coreDeviceIdentifier: .identifier,
      xcodeIdentifier: .hardwareProperties.udid,
      name: (.deviceProperties.name // .name // ""),
      tunnelState: (.connectionProperties.tunnelState // "unknown"),
      transportType: (.connectionProperties.transportType // "unknown")
    }
]
