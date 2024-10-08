export function isSteamGame(overview: any): boolean {
    const hasOwnerAccountId = overview.owner_account_id !== undefined;
    const wasPurchased = !!overview.rt_purchased_time;
    const hasSize = overview.size_on_disk !== '0';

    return hasOwnerAccountId || wasPurchased || hasSize;
}
