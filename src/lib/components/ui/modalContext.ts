import { createContext } from "svelte";

/** The id ModalHeader puts on its title, which Modal names itself by. */
export const [getModalTitleId, setModalTitleId] = createContext<string>();
