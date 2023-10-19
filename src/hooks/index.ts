import { useQuery, useQueryClient } from "@tanstack/react-query";
import { get_adapters, get_status, scan_devices } from "../queries";
import { useEffect, useState } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ParsedDeviceStatus } from "../types";

export function useAdapters() {
  return useQuery({
    queryKey: ["adapters"],
    queryFn: () => get_adapters(),
    staleTime: Infinity,
  });
}

export function useDevices() {
  const adapters = useAdapters();
  const selectedAdapter = adapters.data?.selected ?? "";

  return useQuery({
    queryKey: ["devices"],
    queryFn: scan_devices,
    enabled: !!selectedAdapter,
  });
}

export function useDeviceStatus(
  id: string | null | undefined,
) {
  useDeviceSubscription(id ?? "");
  return useQuery({
    queryKey: ["devices", id],
    queryFn: () => get_status(id ?? ""),
  });
}
export function useDeviceSubscription(id: string) {
  const queryClient = useQueryClient();

  const [isListening, setIsListening] = useState(false);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    console.log(id)
    async function fetchAndListen() {
      unlisten = await listen<ParsedDeviceStatus>(id, (event) => {
        console.log("event", event);
        queryClient.setQueryData<ParsedDeviceStatus>(
          ["devices", id],
          event.payload,
        );
      });
      setIsListening(true);
    }
    fetchAndListen();

    return () => {
      if (unlisten) unlisten();
      setIsListening(false);
    };
  }, [queryClient, id]);
  
  return { isListening };
}
