import React from 'react'

// Shared modal utilities and components

export interface BaseModalProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}

export interface ModalState {
  isOpen: boolean
  loading: boolean
  error: string | null
}

// Shared modal backdrop click handler
export const handleModalBackdropClick = (e: React.MouseEvent, onClose: () => void) => {
  if (e.target === e.currentTarget) {
    onClose()
  }
}

// Shared modal state hook
export const useModalState = (initialState: Partial<ModalState> = {}) => {
  const [state, setState] = React.useState<ModalState>({
    isOpen: false,
    loading: false,
    error: null,
    ...initialState
  })
  
  const setLoading = (loading: boolean) => setState(prev => ({ ...prev, loading }))
  const setError = (error: string | null) => setState(prev => ({ ...prev, error }))
  const setOpen = (isOpen: boolean) => setState(prev => ({ ...prev, isOpen }))
  const resetState = () => setState(prev => ({ ...prev, loading: false, error: null }))
  
  return {
    ...state,
    setLoading,
    setError,
    setOpen,
    resetState
  }
}

// Base modal wrapper component
export const ModalWrapper: React.FC<{
  isOpen: boolean
  onClose: () => void
  size?: 'sm' | 'md' | 'lg' | 'xl'
  children: React.ReactNode
  className?: string
}> = ({ isOpen, onClose, size = 'md', children, className = '' }) => {
  if (!isOpen) return null
  
  const sizeClass = {
    sm: 'modal--sm',
    md: 'modal--md', 
    lg: 'modal--lg',
    xl: 'modal--xl'
  }[size]
  
  return (
    <div className="modal-backdrop" onClick={(e) => handleModalBackdropClick(e, onClose)}>
      <div className={`modal ${sizeClass} ${className}`}>
        {children}
      </div>
    </div>
  )
}

// Shared modal header component
export const ModalHeader: React.FC<{
  title: string
  onClose: () => void
  icon?: React.ReactNode
}> = ({ title, onClose, icon }) => (
  <div className="modal__header">
    <div className="modal__title-section">
      {icon && <div className="modal__icon">{icon}</div>}
      <h3 className="modal__title">{title}</h3>
    </div>
    <button
      className="btn btn--icon btn--ghost"
      onClick={onClose}
      title="Close"
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
        <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
      </svg>
    </button>
  </div>
)

// Shared modal footer component
export const ModalFooter: React.FC<{
  children: React.ReactNode
}> = ({ children }) => (
  <div className="modal__footer">
    <div className="modal__actions">
      {children}
    </div>
  </div>
)

// Shared error display component
export const ModalError: React.FC<{
  error: string | null
}> = ({ error }) => {
  if (!error) return null
  
  return (
    <div className="error-message">
      <span>{error}</span>
    </div>
  )
}

// Shared loading state component
export const ModalLoading: React.FC<{
  message?: string
}> = ({ message = "Loading..." }) => (
  <div className="loading-state">
    <svg className="spinner" width="24" height="24" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeDasharray="31.416" strokeDashoffset="31.416">
        <animate attributeName="stroke-dasharray" dur="2s" values="0 31.416;15.708 15.708;0 31.416" repeatCount="indefinite"/>
        <animate attributeName="stroke-dashoffset" dur="2s" values="0;-15.708;-31.416" repeatCount="indefinite"/>
      </circle>
    </svg>
    <span>{message}</span>
  </div>
)

// Utility functions for common modal operations
export const ModalUtils = {
  // Format date for modal display
  formatDate: (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  },
  
  // Create modal form field
  createFormField: (
    label: string,
    value: string | number,
    onChange: (value: string) => void,
    options: {
      type?: 'text' | 'number' | 'email' | 'password'
      placeholder?: string
      required?: boolean
      disabled?: boolean
      help?: string
    } = {}
  ) => {
    const {
      type = 'text',
      placeholder,
      required = false,
      disabled = false,
      help
    } = options
    
    return {
      label,
      input: (
        <input
          type={type}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          required={required}
          disabled={disabled}
          className="config-input"
        />
      ),
      help: help && <p className="config-help">{help}</p>
    }
  }
}