// java.beans — the part a course program meets: a property-change listener,
// its event, and the support class a model uses to fire one (and that
// `SwingWorker` reports its progress and state through). Captured from a
// JDK 11 first: an unchanged value fires nothing, the general listeners fire
// before the ones registered for a name, and a listener for a name is listed
// as a `PropertyChangeListenerProxy`.

interface PropertyChangeListener {
  void propertyChange(PropertyChangeEvent evt);
}

class PropertyChangeEvent {
  private final Object __source;
  private final String __propertyName;
  private final Object __oldValue;
  private final Object __newValue;
  private Object __propagationId = null;

  public PropertyChangeEvent(Object source, String propertyName, Object oldValue, Object newValue) {
    if (source == null) throw new IllegalArgumentException("null source");
    __source = source;
    __propertyName = propertyName;
    __oldValue = oldValue;
    __newValue = newValue;
  }

  public Object getSource() { return __source; }
  public String getPropertyName() { return __propertyName; }
  public Object getOldValue() { return __oldValue; }
  public Object getNewValue() { return __newValue; }
  public Object getPropagationId() { return __propagationId; }
  public void setPropagationId(Object propagationId) { __propagationId = propagationId; }

  public String toString() {
    return getClass().getName() + "[propertyName=" + __propertyName + "; oldValue=" + __oldValue
        + "; newValue=" + __newValue + "; propagationId=" + __propagationId + "; source="
        + __source + "]";
  }
}

// A listener registered for one property, as `getPropertyChangeListeners()`
// lists it.
class PropertyChangeListenerProxy implements PropertyChangeListener {
  private final String __propertyName;
  private final PropertyChangeListener __listener;
  public PropertyChangeListenerProxy(String propertyName, PropertyChangeListener listener) {
    __propertyName = propertyName;
    __listener = listener;
  }
  public String getPropertyName() { return __propertyName; }
  public PropertyChangeListener getListener() { return __listener; }
  public void propertyChange(PropertyChangeEvent evt) { __listener.propertyChange(evt); }
}

class PropertyChangeSupport {
  private final Object __source;
  private java.util.ArrayList<PropertyChangeListener> __common =
      new java.util.ArrayList<PropertyChangeListener>();
  // Listeners for one property: the names and listeners side by side, in the
  // order they were added.
  private java.util.ArrayList<String> __names = new java.util.ArrayList<String>();
  private java.util.ArrayList<PropertyChangeListener> __named =
      new java.util.ArrayList<PropertyChangeListener>();

  public PropertyChangeSupport(Object sourceBean) {
    if (sourceBean == null) throw new NullPointerException();
    __source = sourceBean;
  }

  public synchronized void addPropertyChangeListener(PropertyChangeListener listener) {
    if (listener == null) return;
    if (listener instanceof PropertyChangeListenerProxy) {
      PropertyChangeListenerProxy proxy = (PropertyChangeListenerProxy) listener;
      addPropertyChangeListener(proxy.getPropertyName(), proxy.getListener());
    } else {
      __common.add(listener);
    }
  }

  public synchronized void removePropertyChangeListener(PropertyChangeListener listener) {
    if (listener == null) return;
    if (listener instanceof PropertyChangeListenerProxy) {
      PropertyChangeListenerProxy proxy = (PropertyChangeListenerProxy) listener;
      removePropertyChangeListener(proxy.getPropertyName(), proxy.getListener());
    } else {
      __common.remove(listener);
    }
  }

  public synchronized void addPropertyChangeListener(String propertyName, PropertyChangeListener listener) {
    if (listener == null || propertyName == null) return;
    __names.add(propertyName);
    __named.add(listener);
  }

  public synchronized void removePropertyChangeListener(String propertyName, PropertyChangeListener listener) {
    if (listener == null || propertyName == null) return;
    for (int i = 0; i < __names.size(); i++) {
      if (__names.get(i).equals(propertyName) && __named.get(i) == listener) {
        __names.remove(i);
        __named.remove(i);
        return;
      }
    }
  }

  public synchronized PropertyChangeListener[] getPropertyChangeListeners() {
    PropertyChangeListener[] all = new PropertyChangeListener[__common.size() + __named.size()];
    int at = 0;
    for (int i = 0; i < __common.size(); i++) all[at++] = __common.get(i);
    for (int i = 0; i < __named.size(); i++) {
      all[at++] = new PropertyChangeListenerProxy(__names.get(i), __named.get(i));
    }
    return all;
  }

  public synchronized PropertyChangeListener[] getPropertyChangeListeners(String propertyName) {
    java.util.ArrayList<PropertyChangeListener> found = new java.util.ArrayList<PropertyChangeListener>();
    for (int i = 0; i < __names.size(); i++) {
      if (__names.get(i).equals(propertyName)) found.add(__named.get(i));
    }
    PropertyChangeListener[] out = new PropertyChangeListener[found.size()];
    for (int i = 0; i < out.length; i++) out[i] = found.get(i);
    return out;
  }

  // Any general listener counts, whatever the name.
  public synchronized boolean hasListeners(String propertyName) {
    if (!__common.isEmpty()) return true;
    if (propertyName == null) return false;
    for (int i = 0; i < __names.size(); i++) {
      if (__names.get(i).equals(propertyName)) return true;
    }
    return false;
  }

  public void firePropertyChange(String propertyName, Object oldValue, Object newValue) {
    if (oldValue == null || newValue == null || !oldValue.equals(newValue)) {
      firePropertyChange(new PropertyChangeEvent(__source, propertyName, oldValue, newValue));
    }
  }

  public void firePropertyChange(String propertyName, int oldValue, int newValue) {
    if (oldValue != newValue) {
      firePropertyChange(propertyName, Integer.valueOf(oldValue), Integer.valueOf(newValue));
    }
  }

  public void firePropertyChange(String propertyName, boolean oldValue, boolean newValue) {
    if (oldValue != newValue) {
      firePropertyChange(propertyName, Boolean.valueOf(oldValue), Boolean.valueOf(newValue));
    }
  }

  // The general listeners first, then the ones for this property.
  public void firePropertyChange(PropertyChangeEvent event) {
    Object oldValue = event.getOldValue();
    Object newValue = event.getNewValue();
    if (oldValue != null && newValue != null && oldValue.equals(newValue)) return;
    java.util.ArrayList<PropertyChangeListener> targets = new java.util.ArrayList<PropertyChangeListener>();
    synchronized (this) {
      targets.addAll(__common);
      String name = event.getPropertyName();
      if (name != null) {
        for (int i = 0; i < __names.size(); i++) {
          if (__names.get(i).equals(name)) targets.add(__named.get(i));
        }
      }
    }
    for (int i = 0; i < targets.size(); i++) targets.get(i).propertyChange(event);
  }
}
