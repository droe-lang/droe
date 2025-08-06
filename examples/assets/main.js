/**
 * Roelang Main JavaScript Library
 * Provides core functionality for all Roelang web applications
 */

// Global namespace for Roelang
window.Roelang = window.Roelang || {};

// Data store management
Roelang.DataStore = {
  // Get storage based on type
  getStorage: function(storageType) {
    return storageType === 'short_store' ? sessionStorage : localStorage;
  },
  
  // Save data model to storage
  save: function(modelName, data, storageType) {
    if (!storageType) return;
    
    try {
      const storage = this.getStorage(storageType);
      storage.setItem(`roelang_${modelName}`, JSON.stringify(data));
    } catch (e) {
      console.error('Failed to save data:', e);
    }
  },
  
  // Load data model from storage
  load: function(modelName, storageType) {
    if (!storageType) return null;
    
    try {
      const storage = this.getStorage(storageType);
      const data = storage.getItem(`roelang_${modelName}`);
      return data ? JSON.parse(data) : null;
    } catch (e) {
      console.error('Failed to load data:', e);
      return null;
    }
  },
  
  // Clear data from storage
  clear: function(modelName, storageType) {
    if (!storageType) return;
    
    try {
      const storage = this.getStorage(storageType);
      storage.removeItem(`roelang_${modelName}`);
    } catch (e) {
      console.error('Failed to clear data:', e);
    }
  }
};

// Validation functions
Roelang.Validation = {
  required: function(value) {
    return value !== null && value !== undefined && value.toString().trim() !== '';
  },
  
  email: function(value) {
    if (!value) return true; // Empty is valid, use required for mandatory
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(value);
  },
  
  numeric: function(value) {
    if (!value) return true;
    return !isNaN(value) && isFinite(value);
  },
  
  minLength: function(value, length) {
    if (!value) return true;
    return value.toString().length >= length;
  },
  
  maxLength: function(value, length) {
    if (!value) return true;
    return value.toString().length <= length;
  },
  
  pattern: function(value, pattern) {
    if (!value) return true;
    const regex = new RegExp(pattern);
    return regex.test(value);
  }
};

// UI Helper functions
Roelang.UI = {
  showError: function(elementId, message) {
    const element = document.getElementById(elementId);
    const errorElement = document.getElementById(elementId + '-error');
    if (element && errorElement) {
      element.classList.add('input-error');
      errorElement.textContent = message;
      errorElement.style.display = 'block';
    }
  },
  
  clearError: function(elementId) {
    const element = document.getElementById(elementId);
    const errorElement = document.getElementById(elementId + '-error');
    if (element && errorElement) {
      element.classList.remove('input-error');
      errorElement.style.display = 'none';
    }
  },
  
  showNotification: function(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification ${type} slide-in`;
    notification.textContent = message;
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 1rem 1.5rem;
      background: ${type === 'success' ? '#28a745' : type === 'error' ? '#dc3545' : '#17a2b8'};
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
  }
};

// Data binding system
Roelang.DataBinding = {
  models: {},
  bindings: {},
  
  // Create a reactive data model
  createModel: function(modelName, ModelClass, storageType) {
    const instance = new ModelClass();
    
    // Load existing data from storage
    if (storageType) {
      const savedData = Roelang.DataStore.load(modelName, storageType);
      if (savedData) {
        Object.assign(instance, savedData);
      }
    }
    
    // Create a proxy to track changes
    const proxy = new Proxy(instance, {
      set: (target, property, value) => {
        target[property] = value;
        
        // Update UI elements bound to this property
        const bindingKey = `${modelName}.${property}`;
        if (this.bindings[bindingKey]) {
          this.bindings[bindingKey].forEach(elementId => {
            this.updateElement(elementId, value);
          });
        }
        
        // Save to storage if configured
        if (storageType) {
          Roelang.DataStore.save(modelName, target, storageType);
        }
        
        return true;
      }
    });
    
    this.models[modelName] = proxy;
    return proxy;
  },
  
  // Bind an element to a data property
  bind: function(elementId, bindingPath) {
    if (!this.bindings[bindingPath]) {
      this.bindings[bindingPath] = [];
    }
    this.bindings[bindingPath].push(elementId);
    
    // Set up two-way binding
    const element = document.getElementById(elementId);
    if (element) {
      const [modelName, property] = bindingPath.split('.');
      const model = this.models[modelName];
      
      if (model) {
        // Update element with current model value
        this.updateElement(elementId, model[property]);
        
        // Listen for element changes
        element.addEventListener('input', function() {
          const value = element.type === 'checkbox' ? element.checked : element.value;
          model[property] = value;
        });
      }
    }
  },
  
  // Update element value
  updateElement: function(elementId, value) {
    const element = document.getElementById(elementId);
    if (element) {
      if (element.type === 'checkbox' || element.type === 'radio') {
        element.checked = value;
      } else {
        element.value = value || '';
      }
    }
  }
};

// Form handling
Roelang.Forms = {
  // Collect form data with validation
  collectFormData: function(formId, validationRules = {}) {
    const form = document.getElementById(formId);
    if (!form) {
      console.error('Form not found:', formId);
      return null;
    }
    
    const formData = {};
    const errors = {};
    let isValid = true;
    
    // Collect all form elements
    const elements = form.querySelectorAll('input, textarea, select');
    
    elements.forEach(element => {
      const id = element.id;
      const name = element.name || id;
      
      // Get value based on element type
      let value;
      if (element.type === 'checkbox') {
        value = element.checked;
      } else if (element.type === 'radio') {
        if (element.checked) {
          formData[name] = element.value;
        }
        return; // Skip to next element
      } else if (element.type === 'number') {
        value = element.value ? parseFloat(element.value) : null;
      } else {
        value = element.value;
      }
      
      formData[name] = value;
      
      // Apply validation rules
      if (validationRules[id]) {
        const rules = validationRules[id];
        
        for (const rule of rules) {
          if (rule.type === 'required' && !Roelang.Validation.required(value)) {
            errors[id] = rule.message || 'This field is required';
            Roelang.UI.showError(id, errors[id]);
            isValid = false;
            break;
          } else if (rule.type === 'email' && !Roelang.Validation.email(value)) {
            errors[id] = rule.message || 'Please enter a valid email';
            Roelang.UI.showError(id, errors[id]);
            isValid = false;
            break;
          } else if (rule.type === 'numeric' && !Roelang.Validation.numeric(value)) {
            errors[id] = rule.message || 'Please enter a valid number';
            Roelang.UI.showError(id, errors[id]);
            isValid = false;
            break;
          } else {
            Roelang.UI.clearError(id);
          }
        }
      }
    });
    
    return {
      data: formData,
      errors: errors,
      isValid: isValid
    };
  },
  
  // Submit form data
  submitForm: function(formId, actionName, validationRules = {}) {
    const result = this.collectFormData(formId, validationRules);
    
    if (!result || !result.isValid) {
      console.log('Form validation failed');
      return false;
    }
    
    // Call the action function if it exists
    if (typeof window[actionName] === 'function') {
      window[actionName](result.data);
    } else {
      console.error('Action not found:', actionName);
    }
    
    return true;
  }
};

// Initialize Roelang when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
  // Auto-initialize data binding
  if (window.RoelangInit && typeof window.RoelangInit.initializeBindings === 'function') {
    window.RoelangInit.initializeBindings();
  }
  
  // Set up form validation on all required fields
  document.querySelectorAll('[required]').forEach(element => {
    element.addEventListener('blur', function() {
      if (!Roelang.Validation.required(this.value)) {
        Roelang.UI.showError(this.id, 'This field is required');
      } else {
        Roelang.UI.clearError(this.id);
      }
    });
  });
  
  // Set up email validation
  document.querySelectorAll('[type="email"]').forEach(element => {
    element.addEventListener('blur', function() {
      if (this.value && !Roelang.Validation.email(this.value)) {
        Roelang.UI.showError(this.id, 'Please enter a valid email address');
      } else {
        Roelang.UI.clearError(this.id);
      }
    });
  });
  
  console.log('Roelang application initialized');
});

// Export for use in generated code
window.showError = Roelang.UI.showError;
window.clearError = Roelang.UI.clearError;
window.validateRequired = Roelang.Validation.required;
window.validateEmail = Roelang.Validation.email;