import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { useMapIgnoredSystems } from '@/composables/useMapIgnoredSystems';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { useShowMap } from '@/composables/useShowMap';
import { useStaticData } from '@/composables/useStaticData';
import { useTrackingSystems } from '@/composables/useTrackingSystems';
import { aliasTargetKind, suggestAlias } from '@/lib/alias';
import { buildSignatureBookmark } from '@/lib/bookmark';
import { groupSignatureOptions } from '@/lib/signatureCompatibility';
import { isWormholeSystem } from '@/lib/solarsystem';
import { createTracking, updateMapUserSettings, useMapSolarsystems } from '@/map/api';
import { TLifetimeStatus, TMassStatus, TShipSize, TSignature } from '@/types/models';
import { computed, ref, watch } from 'vue';
import { toast } from 'vue-sonner';

export function useTracking() {
    const character = useActiveMapCharacter();
    const map_user_settings = useMapUserSettings();
    const { isIgnored } = useMapIgnoredSystems();
    const page = useShowMap();
    const { staticData } = useStaticData();
    const { map_solarsystems } = useMapSolarsystems();

    const is_tracking = computed(() => map_user_settings.value?.is_tracking && character.value && map_user_settings.value?.tracking_allowed);
    const is_tracking_allowed = computed(() => map_user_settings.value.tracking_allowed);
    const can_track = computed(() => character.value && map_user_settings.value.tracking_allowed);

    const { origin_map_solarsystem, target_solarsystem, update } = useTrackingSystems();

    const show_signature_modal = ref(false);
    // All of the origin's signatures; the dialog demotes the ones that cannot
    // lead to the target instead of hiding them.
    const signatures = computed(() => origin_map_solarsystem.value?.signatures?.toSorted(sortSignatures));
    const possible_signatures = computed(() => groupSignatureOptions(signatures.value ?? [], target_solarsystem.value?.class).likely);
    const existing_map_solarsystem = computed(() => map_solarsystems.value.find((s) => s.solarsystem_id === target_solarsystem.value?.id));
    const existing_connection = computed(() => {
        if (!existing_map_solarsystem.value) return null;
        return (
            signatures.value?.find(
                (s) =>
                    s.map_connection?.to_map_solarsystem_id === existing_map_solarsystem.value?.id ||
                    s.map_connection?.from_map_solarsystem_id === existing_map_solarsystem.value?.id,
            ) || null
        );
    });

    const known_aliases = computed(() => map_solarsystems.value.map((s) => s.alias).filter((alias): alias is string => Boolean(alias)));

    // Pre-fill the signature dialog's alias field. An alias the target already
    // carries on the map wins; otherwise we guess the next chain alias.
    const suggested_alias = computed(() => {
        if (existing_map_solarsystem.value?.alias) {
            return existing_map_solarsystem.value.alias;
        }

        if (!map_user_settings.value.suggest_alias_enabled) return null;

        const origin = origin_map_solarsystem.value;
        const target = target_solarsystem.value;
        if (!origin || !target) return null;

        const targetIsWormhole = isWormholeSystem(target);

        return suggestAlias({
            parentAlias: origin.alias,
            targetIsWormhole,
            originIsWormhole: isWormholeSystem(origin.solarsystem),
            aliases: known_aliases.value,
            scheme: page.props.map.bookmark_alias_scheme,
            targetKind: aliasTargetKind(targetIsWormhole, target.class),
        });
    });

    watch(
        () => [character.value?.id, character.value?.status?.solarsystem_id] as const,
        ([new_character_id, new_solarsystem_id], [old_character_id, old_solarsystem_id]) => {
            if (!map_user_settings.value.is_tracking) return;
            if (!new_solarsystem_id || !old_solarsystem_id) return;
            if (new_solarsystem_id === old_solarsystem_id) return;
            // Only a single character moving between systems is a real jump. When
            // the active character is switched the watched system id also changes,
            // but that must not create a connection between the two characters' systems.
            if (new_character_id !== old_character_id) return;

            handleSolarsystemJump(old_solarsystem_id, new_solarsystem_id);
        },
    );

    function handleSolarsystemJump(old_solarsystem_id: number | null, new_solarsystem_id: number) {
        if (isIgnored(new_solarsystem_id)) return;
        const old_map_solarsystem = map_solarsystems.value.find((s) => s.solarsystem_id === old_solarsystem_id);
        if (!old_map_solarsystem) return;
        if (old_map_solarsystem.solarsystem_id === new_solarsystem_id) return;
        update(old_map_solarsystem.solarsystem_id, new_solarsystem_id, performJump);
    }

    function isGateConnected(origin_solarsystem_id: number | null | undefined, target_solarsystem_id: number | null | undefined): boolean {
        if (!origin_solarsystem_id || !target_solarsystem_id) return false;
        return staticData.value?.connections[origin_solarsystem_id]?.includes(target_solarsystem_id) ?? false;
    }

    function sortSignatures(a: TSignature, b: TSignature) {
        if (!a.signature_id || !b.signature_id) return 0;
        return a.signature_id.localeCompare(b.signature_id);
    }

    function performJump() {
        if (existing_connection.value?.map_connection_id) return;
        const gate_connected = isGateConnected(origin_map_solarsystem.value?.solarsystem_id, target_solarsystem.value?.id);
        if (gate_connected || !possible_signatures.value.length || !map_user_settings.value.prompt_for_signature_enabled) {
            return createTracking(origin_map_solarsystem.value!.id, target_solarsystem.value!.id);
        }

        show_signature_modal.value = true;
    }

    function handleToggle() {
        if (!map_user_settings.value.tracking_allowed) return;

        updateMapUserSettings(page.props.map.slug, {
            is_tracking: !map_user_settings.value.is_tracking,
        });
    }

    function handleSelectSignature(selection: {
        signatureId: number | null;
        alias: string | null;
        lifetime: TLifetimeStatus;
        massStatus: TMassStatus;
        shipSize: TShipSize | null;
    }) {
        show_signature_modal.value = false;
        if (!origin_map_solarsystem.value || !target_solarsystem.value) return;
        copyConnectionBookmark(selection.signatureId, selection.alias);
        createTracking(origin_map_solarsystem.value.id, target_solarsystem.value.id, {
            signature_id: selection.signatureId,
            alias: selection.alias,
            lifetime: selection.lifetime,
            mass_status: selection.massStatus,
            ship_size: selection.shipSize,
        });
    }

    // Copy the connection bookmark for the system we just jumped into, using the
    // same scheme as the connection context menu: the current system labelled
    // with the signature we used in the origin. Delegates to the shared
    // signature-bookmark builder, so an empty chosen alias now falls back to
    // the guessed one instead of leaving the alias token blank.
    function copyConnectionBookmark(signatureId: number | null, alias: string | null) {
        if (!map_user_settings.value.copy_bookmark_enabled) return;
        const target = target_solarsystem.value;
        if (!target) return;

        const signature = signatures.value?.find((s) => s.id === signatureId) ?? null;
        const name = buildSignatureBookmark({
            signature: {
                signature_id: signature?.signature_id ?? null,
                ship_size: signature?.ship_size ?? null,
                mass_status: signature?.mass_status ?? null,
                lifetime: signature?.lifetime ?? 'healthy',
                wormhole: signature?.wormhole,
                signature_type: signature?.signature_type,
            },
            currentSystem: { alias: origin_map_solarsystem.value?.alias },
            connectionTarget: { alias, occupier_alias: existing_map_solarsystem.value?.occupier_alias, solarsystem: target },
            aliases: known_aliases.value,
            formats: page.props.map,
        });

        navigator.clipboard.writeText(name);
        toast.success('Copied bookmark to clipboard', { description: name });
    }

    return {
        toggle: handleToggle,
        is_tracking,
        is_tracking_allowed,
        can_track,
        signatures,
        show_signature_modal,
        handleSelectSignature,
        origin_map_solarsystem,
        target_solarsystem,
        existing_map_solarsystem,
        suggested_alias,
    };
}
