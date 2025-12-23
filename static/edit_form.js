function confirmDelete() {
    if (confirm('Are you sure you want to delete this code? This action cannot be undone.')) {
        const deleteForm = document.getElementById('deleteForm');
        if (!deleteForm){
            console.error("Delete form not found!");
        } else {
            deleteForm.submit();
        };
    }
}