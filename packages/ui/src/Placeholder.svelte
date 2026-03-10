<script lang="ts">
  import logoSvg from './lib/nisaba-logo.svg?url';

  let { message = 'Select a note' } = $props<{ message?: string }>();
</script>

<div class="placeholder">
  <img src={logoSvg} alt="" class="logo" />
  <span class="message">{message}</span>
</div>

<style>
  .placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: #52525b;
    font-size: 14px;
  }

  .logo {
    width: 192px;
    height: 192px;
    object-fit: contain;
    opacity: 0.7;
  }

  .message {
    color: inherit;
  }
</style>
