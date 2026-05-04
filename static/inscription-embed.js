document.addEventListener('click', async (event) => {
  const button = event.target.closest('[data-embed-copy]');
  if (!button) return;

  const id = button.getAttribute('data-inscription-id');
  if (!id) return;

  const width = button.getAttribute('data-width') || '480';
  const height = button.getAttribute('data-height') || '120';
  const origin = window.location.origin;
  const snippet =
    `<iframe src="${origin}/embed/${id}" width="${width}" height="${height}" ` +
    `frameborder="0" allowfullscreen></iframe>`;

  try {
    await navigator.clipboard.writeText(snippet);
    const previousText = button.textContent;
    button.textContent = 'copied!';
    button.disabled = true;
    setTimeout(() => {
      button.textContent = previousText;
      button.disabled = false;
    }, 1500);
  } catch (err) {
    console.error('embed copy failed', err);
  }
});
