<script lang="ts">
    import { Select as SelectPrimitive } from "bits-ui";

    let { placeholder = "", ...restProps } = $props();
</script>

<SelectPrimitive.Value {placeholder} data-slot="select-value" {...restProps} />
