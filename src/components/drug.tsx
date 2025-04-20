import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";


export interface Props {
    onFileDrop: (insertStr: string) => void;
}


export const DrugComponent = (props: Props) => {
    const handleFileToString = async (filePaths: string[]): Promise<string> => {
        try {
            const res = await invoke('files_to_string', { filepaths: filePaths });
            console.log(res);
            return res as string;
        } catch (e) {
            console.error(e);
            return "";
        }
    };

    useEffect(() => {
        let unlisten: UnlistenFn | undefined;

        const setupListener = async () => {
            unlisten = await listen<{ paths: string[] }>('tauri://drag-drop', async (event) => {
                const filepath = event.payload.paths;
                if (filepath && filepath.length > 0) {
                    const str = await handleFileToString(filepath);
                    console.debug('load file:', filepath, str);
                    props.onFileDrop(str);
                }
            });
        };

        setupListener();

        return () => {
            if (unlisten) {
                unlisten();
            }
        };
    }, []);

    return (
        <></>
    );
}