// Custom JavaScript for Roelang Applications

// Enhanced form validation
function enhancedValidation(formId) {
  const form = document.getElementById(formId);
  if (!form) return;
  
  // Add real-time validation
  form.addEventListener('input', function(e) {
    if (e.target.hasAttribute('required')) {
      if (e.target.value.trim() === '') {
        e.target.classList.add('input-error');
      } else {
        e.target.classList.remove('input-error');
      }
    }
  });
}

// Image gallery lightbox
function initializeGallery() {
  const images = document.querySelectorAll('.img-fluid, .img-thumbnail');
  
  images.forEach(img => {
    img.addEventListener('click', function() {
      // Create lightbox overlay
      const overlay = document.createElement('div');
      overlay.className = 'layout-overlay fade-in';
      overlay.style.cursor = 'pointer';
      
      const largeImg = document.createElement('img');
      largeImg.src = this.src;
      largeImg.alt = this.alt;
      largeImg.style.maxWidth = '90%';
      largeImg.style.maxHeight = '90%';
      largeImg.className = 'slide-in';
      
      overlay.appendChild(largeImg);
      document.body.appendChild(overlay);
      
      // Close on click
      overlay.addEventListener('click', function() {
        document.body.removeChild(overlay);
      });
    });
  });
}

// Initialize custom behaviors
document.addEventListener('DOMContentLoaded', function() {
  console.log('Roelang application initialized');
  
  // Initialize gallery if images exist
  if (document.querySelector('.img-fluid, .img-thumbnail')) {
    initializeGallery();
  }
  
  // Add smooth scrolling
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute('href'));
      if (target) {
        target.scrollIntoView({ behavior: 'smooth' });
      }
    });
  });
  
  // Add loading states to buttons
  document.querySelectorAll('.button').forEach(button => {
    button.addEventListener('click', function() {
      if (!this.classList.contains('loading')) {
        this.classList.add('loading');
        setTimeout(() => {
          this.classList.remove('loading');
        }, 1000);
      }
    });
  });
});

// Export for use in Roelang actions
window.RoelangUtils = {
  showNotification: function(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification ${type} slide-in`;
    notification.textContent = message;
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 1rem 1.5rem;
      background: ${type === 'success' ? '#28a745' : '#17a2b8'};
      color: white;
      border-radius: 4px;
      box-shadow: 0 2px 10px rgba(0,0,0,0.1);
      z-index: 9999;
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
      notification.style.opacity = '0';
      setTimeout(() => document.body.removeChild(notification), 300);
    }, 3000);
  },
  
  validateForm: function(formId) {
    const form = document.getElementById(formId);
    if (!form) return false;
    
    const inputs = form.querySelectorAll('[required]');
    let isValid = true;
    
    inputs.forEach(input => {
      if (!input.value.trim()) {
        input.classList.add('input-error');
        isValid = false;
      }
    });
    
    return isValid;
  }
};