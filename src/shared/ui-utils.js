/**
 * Shared UI Utilities Module
 * Common UI functions and form helpers
 */

/**
 * UI Utilities Class
 */
export class UIUtils {
  /**
   * Add event listeners to time inputs with validation
   */
  static setupTimeInputs(inputs, onChangeFn = null) {
    inputs.forEach(({ element, max }) => {
      element.addEventListener('blur', () => {
        this.validateTimeInput(element, max);
        if (onChangeFn) onChangeFn();
      });
      
      element.addEventListener('input', () => {
        if (element.value.length > 3) {
          element.value = element.value.slice(0, 3);
        }
      });
    });
  }

  /**
   * Validate time input values
   */
  static validateTimeInput(input, maxValue) {
    let value = parseInt(input.value, 10);
    if (isNaN(value) || value < 0) {
      input.value = 0;
    } else if (value > maxValue) {
      input.value = maxValue;
    } else {
      input.value = value;
    }
  }

  /**
   * Setup form change listeners
   */
  static setupFormListeners(formId, onChangeFn) {
    const form = document.getElementById(formId);
    if (form) {
      form.addEventListener('change', onChangeFn);
    }
  }

  /**
   * Apply settings to UI elements
   */
  static applySettingsToForm(settings, fieldMappings) {
    Object.entries(fieldMappings).forEach(([settingKey, elementId]) => {
      const element = document.getElementById(elementId);
      if (!element) return;

      const value = settings[settingKey];
      
      if (element.type === 'checkbox') {
        element.checked = value;
      } else if (element.type === 'radio') {
        if (element.value === value) {
          element.checked = true;
        }
      } else {
        element.value = value;
      }
    });
  }

  /**
   * Get form values as object
   */
  static getFormValues(fieldMappings) {
    const values = {};
    
    Object.entries(fieldMappings).forEach(([settingKey, elementId]) => {
      const element = document.getElementById(elementId);
      if (!element) return;

      if (element.type === 'checkbox') {
        values[settingKey] = element.checked;
      } else if (element.type === 'radio') {
        if (element.checked) {
          values[settingKey] = element.value;
        }
      } else if (element.type === 'number') {
        values[settingKey] = parseInt(element.value) || 0;
      } else {
        values[settingKey] = element.value;
      }
    });

    return values;
  }

  /**
   * Show/hide elements based on condition
   */
  static toggleElementVisibility(elementId, show) {
    const element = document.getElementById(elementId);
    if (element) {
      element.style.display = show ? 'block' : 'none';
    }
  }

  /**
   * Add/remove CSS classes
   */
  static toggleClass(elementId, className, add) {
    const element = document.getElementById(elementId);
    if (element) {
      if (add) {
        element.classList.add(className);
      } else {
        element.classList.remove(className);
      }
    }
  }

  /**
   * Update text content of element
   */
  static updateText(elementId, text) {
    const element = document.getElementById(elementId);
    if (element) {
      element.textContent = text;
    }
  }

  /**
   * Setup keyboard shortcuts
   */
  static setupKeyboardShortcuts(shortcuts) {
    document.addEventListener('keydown', (e) => {
      const shortcut = shortcuts.find(s => 
        s.key === e.key && 
        (s.ctrl === undefined || s.ctrl === e.ctrlKey) &&
        (s.alt === undefined || s.alt === e.altKey) &&
        (s.shift === undefined || s.shift === e.shiftKey)
      );
      
      if (shortcut) {
        e.preventDefault();
        shortcut.action();
      }
    });
  }

  /**
   * Setup drag functionality for windows
   */
  static setupDragFunctionality(containerElement, onDragStart = null, onDragEnd = null) {
    let isDragging = false;
    let dragOffset = { x: 0, y: 0 };

    containerElement.addEventListener('mousedown', (e) => {
      // Skip if clicking on interactive elements
      if (e.target.tagName === 'BUTTON' || e.target.tagName === 'INPUT') return;
      
      isDragging = true;
      dragOffset.x = e.clientX;
      dragOffset.y = e.clientY;
      containerElement.style.cursor = 'grabbing';
      
      if (onDragStart) onDragStart();
    });

    document.addEventListener('mousemove', async (e) => {
      if (!isDragging) return;

      const deltaX = e.clientX - dragOffset.x;
      const deltaY = e.clientY - dragOffset.y;

      try {
        const { getCurrentWindow } = window.__TAURI__.window;
        const window = getCurrentWindow();
        const currentPos = await window.outerPosition();
        await window.setPosition({
          x: currentPos.x + deltaX,
          y: currentPos.y + deltaY
        });
      } catch (error) {
        console.error('Error during drag:', error);
      }

      dragOffset.x = e.clientX;
      dragOffset.y = e.clientY;
    });

    document.addEventListener('mouseup', () => {
      if (isDragging) {
        isDragging = false;
        containerElement.style.cursor = 'grab';
        
        if (onDragEnd) onDragEnd();
      }
    });

    containerElement.style.cursor = 'grab';
  }
}

/**
 * Tab Navigation Manager
 */
export class TabManager {
  constructor() {
    this.tabs = [];
    this.contents = [];
    this.activeTab = null;
  }

  /**
   * Initialize tab navigation
   */
  init() {
    this.tabs = document.querySelectorAll('.tab');
    this.contents = document.querySelectorAll('.tab-content');

    this.tabs.forEach(tab => {
      tab.addEventListener('click', () => {
        this.switchTab(tab.dataset.tab);
      });
    });
  }

  /**
   * Switch to specific tab
   */
  switchTab(targetTab) {
    // Update active tab
    this.tabs.forEach(tab => tab.classList.remove('active'));
    const activeTab = document.querySelector(`[data-tab="${targetTab}"]`);
    if (activeTab) {
      activeTab.classList.add('active');
      this.activeTab = targetTab;
    }

    // Show target content
    this.contents.forEach(content => content.classList.remove('active'));
    const targetContent = document.getElementById(`${targetTab}-tab`);
    if (targetContent) {
      targetContent.classList.add('active');
    }
  }

  /**
   * Get current active tab
   */
  getActiveTab() {
    return this.activeTab;
  }
}