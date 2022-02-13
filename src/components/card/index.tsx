import { motion } from "framer-motion";

const Card = () => {
    return (
        <motion.div className="w-full h-full rounded-xl bg-white opacity-30"
            initial={{ x: '-100vw' }} animate={{ x: 0 }}
            whileInView="visible"
            viewport={{ once: true }}>
            Arroz
        </motion.div>
    );
}

export default Card;