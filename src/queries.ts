import { invoke } from "@tauri-apps/api";
import {
  AdapterResult,
  Command,
  ParsedDeviceStatus,
  PeripheralResult,
} from "./types";

export async function get_adapters(): Promise<AdapterResult> {
  return await invoke("get_btle_adapters");
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

export async function get_status(id: string): Promise<ParsedDeviceStatus | undefined> {
  return await invoke("get_status", { id });
}
