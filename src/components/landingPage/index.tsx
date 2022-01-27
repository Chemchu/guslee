import { motion } from "framer-motion";

const LandingPage = () => {
    return (
        // <FondoImagen />

        <FondoVideo />

    );
}

const FondoVideo = () => {
    return (
        <motion.div className="items-center font-sans overflow-hidden" >
            <video autoPlay loop muted className='w-full h-full object-cover fixed -z-10'>
                <source src={'/video/tv.mp4'} type="video/mp4" />
            </video>
            <div className="text-white p-4 text-left items-end">
                <div className="flex text-3xl md:text-5xl lg:text-7xl">
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 0 } }}
                        className="font-outfit mr-2 md:mr-4">
                        ¡Hola!
                    </motion.div>
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1 } }}
                        className="font-outfit-thin">
                        Me llamo Gustavo Lee
                    </motion.div>
                </div>
                <div className="text-lg md:text-3xl lg:text-5xl my-4">
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1.8 } }}
                        className="font-outfit-thin mr-2 md:mr-4">
                        Soy ingeniero de software
                    </motion.div>
                </div>

            </div>
        </motion.div >
    );
}

const FondoImagen = () => {
    return (
        <main className="bg-main bg-cover bg-no-repeat bg-center font-outfit-light w-screen h-screen p-2 overflow-hidden">
            <div className="text-white m-4 text-left items-end">
                <div className="flex text-3xl md:text-5xl lg:text-7xl">
                    <motion.div initial={{ opacity: 0, x: '100vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 0 } }}
                        className="font-outfit mr-2 md:mr-4">
                        ¡Hola!
                    </motion.div>
                    <motion.div initial={{ opacity: 0, x: '100vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1 } }}
                        className="font-outfit-thin">
                        Me llamo Gustavo Lee
                    </motion.div>
                </div>
                <div className="text-lg md:text-3xl lg:text-5xl my-4">
                    <motion.div initial={{ opacity: 0, x: '100vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1.8 } }}
                        className="font-outfit-thin mr-2 md:mr-4">
                        Soy ingeniero de software
                    </motion.div>
                </div>
            </div>
        </main >
    );
}

export default LandingPage;