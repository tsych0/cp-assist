import { invoke } from "@tauri-apps/api/core";
import { notifications } from "@mantine/notifications";
import { IconCheck, IconX } from "@tabler/icons-react";
import { Language, Problem, Verdict } from "./Languages.ts";

async function invokeWithNotify<T>(
  cmd: string,
  args: Record<string, any> = {},
  successMessage?: string,
  errorMessage?: string,
): Promise<T | null> {
  try {
    const result = await invoke<T>(cmd, args);
    if (successMessage) {
      notifications.show({
        id: `success_${cmd}`,
        message: successMessage,
        icon: <IconCheck size="1.1rem" />,
        color: "teal",
      });
    }
    return result;
  } catch (e) {
    console.error(e);
    notifications.show({
      id: `error_${cmd}`,
      message: errorMessage ? `${errorMessage}: ${e}` : (e as string),
      icon: <IconX size="1.1rem" />,
      color: "red",
    });
    return null;
  }
}

export const set_directory = async (directory: string) =>
  (await invokeWithNotify<boolean>(
    "set_directory",
    { directory },
    undefined,
    "The specified directory was not found",
  )) ?? false;

export const get_directory = async () =>
  (await invokeWithNotify<string>(
    "get_directory",
    {},
    undefined,
    "Cannot get the directory",
  )) ?? "";

export const set_language = async (language_id: number) =>
  (await invokeWithNotify<boolean>(
    "set_language",
    { languageId: language_id },
    "Language set successfully",
    "Could not set language",
  )) ?? false;

export const get_language = async () =>
  (await invokeWithNotify<number>(
    "get_language",
    {},
    undefined,
    "Could not get language",
  )) ?? 0;

export const get_languages = async () =>
  (await invokeWithNotify<Language[]>(
    "get_languages",
    {},
    undefined,
    "Could not get languages",
  )) ?? [];

export const set_problem = async (problem: Problem) =>
  (await invokeWithNotify<boolean>(
    "set_problem",
    { problem },
    undefined,
    "Could not set problem",
  )) ?? false;

export const get_problem = async () =>
  await invokeWithNotify<Problem>(
    "get_problem",
    {},
    undefined,
    "Could not get problem",
  );

export const set_verdicts = async (verdicts: Verdict[]) =>
  (await invokeWithNotify<boolean>(
    "set_verdicts",
    { verdicts },
    undefined,
    "Could not set verdicts",
  )) ?? false;

export const get_verdicts = async () =>
  await invokeWithNotify<Verdict[]>(
    "get_verdicts",
    {},
    undefined,
    "Could not get verdicts",
  );

export const read_config = async () =>
  (await invokeWithNotify<null>(
    "read_config",
    {},
    "Config file succesfully read",
  )) ?? false;

export const create_file = async () =>
  (await invokeWithNotify<boolean>(
    "create_file",
    {},
    "File created",
    "Could not create file",
  )) ?? false;

export const copy_code = async () =>
    (await invokeWithNotify<boolean>(
        "copy_code",
        {},
        "Code copied",
        "Could not copy code",
    )) ?? false;

export const save_state = async () =>
  (await invokeWithNotify<boolean>(
    "save_state",
    {},
    undefined,
    "Could not save state",
  )) ?? false;

export const submit = async () =>
  await invokeWithNotify<null>(
    "submit_solution",
    {},
    "Submitting on codeforces...",
    "Could not submit",
  );

export const run = async () =>
  await invokeWithNotify<null>("test", {}, undefined, "Could not run test");
