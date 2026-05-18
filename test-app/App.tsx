import { View, Text, StyleSheet } from 'react-native';

export default function ProfilePage() {
  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>JD</Text>
        </View>

        <Text style={styles.name}>John Doe</Text>
        <Text style={styles.role}>Mobile Developer</Text>
      </View>

      <View style={styles.card}>
        <Text style={styles.cardTitle}>About</Text>

        <Text style={styles.aboutText}>
          Passionate React Native developer focused on building clean and modern
          mobile applications.
        </Text>
      </View>

      <View style={styles.statsContainer}>
        <View style={styles.statCard}>
          <Text style={styles.statValue}>120</Text>
          <Text style={styles.statLabel}>Posts</Text>
        </View>

        <View style={styles.statCard}>
          <Text style={styles.statValue}>8.5K</Text>
          <Text style={styles.statLabel}>Followers</Text>
        </View>

        <View style={styles.statCard}>
          <Text style={styles.statValue}>340</Text>
          <Text style={styles.statLabel}>Following</Text>
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
    padding: 20,
  },

  header: {
    alignItems: 'center',
    marginTop: 40,
    marginBottom: 30,
  },

  avatar: {
    width: 100,
    height: 100,
    borderRadius: 50,
    backgroundColor: '#d1d5db',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 15,
  },

  avatarText: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#374151',
  },

  name: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#111827',
  },

  role: {
    fontSize: 16,
    color: '#6b7280',
    marginTop: 5,
  },

  card: {
    backgroundColor: '#ffffff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 20,
  },

  cardTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    marginBottom: 15,
    color: '#111827',
  },

  aboutText: {
    fontSize: 15,
    lineHeight: 24,
    color: '#4b5563',
  },

  statsContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },

  statCard: {
    flex: 1,
    backgroundColor: '#ffffff',
    padding: 20,
    borderRadius: 16,
    alignItems: 'center',
    marginHorizontal: 5,
  },

  statValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#111827',
  },

  statLabel: {
    marginTop: 6,
    color: '#6b7280',
  },
});