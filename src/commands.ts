import { invoke } from "@tauri-apps/api/core";
import {
  AdapterResult,
  Command,
  ParsedDeviceStatus,
  PeripheralResult,
  UserPreferences,
} from "./types";
import { Store } from "@tauri-apps/plugin-store";


export async function get_adapters(): Promise<AdapterResult> {
  return invoke("get_btle_adapters");
}

export async function scan_devices(): Promise<PeripheralResult[]> {
  return invoke("scan_devices");
}

export async function connect_device(id: string): Promise<void> {
  return invoke("connect_device", { id });
}

export async function disconnect_device(id: string): Promise<void> {
  return invoke("disconnect_device", { id });
}

export async function send_command(id: string, command: Command) {
  await invoke("send_command", { id, command });
}

export async function get_status(
  id: string,
): Promise<ParsedDeviceStatus | undefined> {
  return invoke("get_status", { id });
}


const store = new Store(".settings.dat");

export async function get_config(): Promise<UserPreferences | null> {
  return store.get("config");
}

export async function set_config(config: UserPreferences): Promise<void> {
    await store.set("config", config);
}
