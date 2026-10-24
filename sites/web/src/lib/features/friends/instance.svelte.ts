// The website's friends store, reactive through Svelte's $state. A change of account forgets the last one's friends.
import { api } from "$lib/api";
import { runeCell, session } from "$lib/data";
import { FriendsStore } from "./friends";

export const friends = new FriendsStore(api.friends, runeCell);

session.onChange((_user, { initial }) => {
  if (!initial) friends.reset();
});
