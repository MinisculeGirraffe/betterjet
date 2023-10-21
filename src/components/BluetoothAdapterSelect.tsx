import { Select } from "@mantine/core";
import { useAdapters } from "../hooks";

export function BluetoothAdapterSelect() {
  const adapters = useAdapters();
  return (
    <Select
      disabled={adapters.isLoading}
      data={adapters.data?.adapters ?? []}
      value={adapters.data?.selected}
      label="Bluetooth Adapter"
      allowDeselect={false}
    />
  )
}